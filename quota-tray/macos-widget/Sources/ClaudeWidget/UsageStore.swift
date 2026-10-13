import Foundation
import ClaudeUsageCore

/// Reads Claude Code's saved sign-in from the Keychain (or the credentials
/// file older installs use) and hands back just the access token.
enum CredentialReader {
    private static let keychainService = "Claude Code-credentials"
    private static let securityBinary = "/usr/bin/security"
    /// `security` exits 44 when the item isn't in the Keychain.
    private static let itemNotFound: Int32 = 44

    static func accessToken() throws -> String {
        try CredentialParser.accessToken(from: try credentialBlob())
    }

    private static func credentialBlob() throws -> Data {
        var keychainError: UsageError?
        do {
            return try readKeychain()
        } catch let error as UsageError {
            keychainError = error
        }
        if let file = readCredentialsFile() {
            return file
        }
        throw keychainError ?? UsageError.credentialsMissing
    }

    private static func readKeychain() throws -> Data {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: securityBinary)
        process.arguments = ["find-generic-password", "-s", keychainService, "-w"]
        let output = Pipe()
        let errors = Pipe()
        process.standardOutput = output
        process.standardError = errors

        do {
            try process.run()
        } catch {
            throw UsageError.credentialsUnreadable(error.localizedDescription)
        }
        let data = output.fileHandleForReading.readDataToEndOfFile()
        let errorText = errors.fileHandleForReading.readDataToEndOfFile()
        process.waitUntilExit()

        if process.terminationStatus == itemNotFound {
            throw UsageError.credentialsMissing
        }
        guard process.terminationStatus == 0 else {
            throw UsageError.credentialsUnreadable(String(decoding: errorText, as: UTF8.self))
        }
        return data
    }

    private static func readCredentialsFile() -> Data? {
        let path = NSHomeDirectory() + "/.claude/.credentials.json"
        return FileManager.default.contents(atPath: path)
    }
}

/// Carries one request's outcome out of the URLSession callback. The semaphore
/// orders the write before the read, so no further locking is needed.
private final class OutcomeBox {
    var result: Result<UsageSnapshot, Error> = .failure(UsageError.network("No response"))
}

/// Polls Anthropic once a minute. A failed poll keeps the last good reading
/// (marked stale) instead of flashing a confident zero.
final class UsageStore: ObservableObject {
    @Published private(set) var snapshot: UsageSnapshot?
    @Published private(set) var lastError: UsageError?

    /// Called on the main thread after every poll, successful or not.
    var onUpdate: (() -> Void)?

    private let endpoint = URL(string: "https://api.anthropic.com/api/oauth/usage")!
    private let pollInterval: TimeInterval = 60
    private let session = URLSession(configuration: .ephemeral)
    private var timer: Timer?
    private var inFlight = false

    func start() {
        refresh()
        let timer = Timer(timeInterval: pollInterval, repeats: true) { [weak self] _ in
            self?.refresh()
        }
        RunLoop.main.add(timer, forMode: .common)
        self.timer = timer
    }

    /// Safe to call from the main thread at any time; overlapping polls collapse into one.
    func refresh() {
        guard !inFlight else { return }
        inFlight = true
        DispatchQueue.global(qos: .utility).async { [weak self] in
            guard let self else { return }
            let result = self.fetchBlocking()
            DispatchQueue.main.async {
                self.inFlight = false
                switch result {
                case .success(let snapshot):
                    self.snapshot = snapshot
                    self.lastError = nil
                case .failure(let error):
                    self.lastError = (error as? UsageError) ?? .network(error.localizedDescription)
                }
                self.onUpdate?()
            }
        }
    }

    /// Runs on a background queue: Keychain read, then one HTTPS request.
    private func fetchBlocking() -> Result<UsageSnapshot, Error> {
        do {
            let token = try CredentialReader.accessToken()
            var request = URLRequest(url: endpoint, timeoutInterval: 20)
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
            request.setValue("oauth-2025-04-20", forHTTPHeaderField: "anthropic-beta")
            request.setValue("application/json", forHTTPHeaderField: "Accept")

            let done = DispatchSemaphore(value: 0)
            let outcome = OutcomeBox()
            session.dataTask(with: request) { data, response, error in
                defer { done.signal() }
                if let error {
                    outcome.result = .failure(UsageError.network(error.localizedDescription))
                    return
                }
                guard let http = response as? HTTPURLResponse, let data else { return }
                switch http.statusCode {
                case 200..<300:
                    outcome.result = Result { try UsageParser.snapshot(from: data) }
                case 401, 403:
                    outcome.result = .failure(UsageError.unauthorized)
                default:
                    outcome.result = .failure(UsageError.http(http.statusCode))
                }
            }.resume()
            done.wait()
            return outcome.result
        } catch {
            return .failure(error)
        }
    }
}

// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "ClaudeWidget",
    platforms: [.macOS(.v13)],
    products: [
        .executable(name: "ClaudeWidget", targets: ["ClaudeWidget"]),
    ],
    targets: [
        // Foundation-only logic (parsing, pace, formatting): builds and tests anywhere.
        .target(name: "ClaudeUsageCore"),
        // The AppKit/SwiftUI app.
        .executableTarget(name: "ClaudeWidget", dependencies: ["ClaudeUsageCore"]),
        .testTarget(name: "ClaudeUsageCoreTests", dependencies: ["ClaudeUsageCore"]),
    ]
)

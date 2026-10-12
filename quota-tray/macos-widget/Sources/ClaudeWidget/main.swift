import AppKit

// Menu-bar-less "accessory" app: no Dock icon, just the floating widget.
let application = NSApplication.shared
let appDelegate = AppDelegate()
application.delegate = appDelegate
application.setActivationPolicy(.accessory)
application.run()

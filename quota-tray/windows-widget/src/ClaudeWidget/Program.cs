using System;
using System.Threading;
using ClaudeWidget;

// One widget per user session: a second launch (for example the login item plus a double-click) just exits.
using var singleInstance = new Mutex(initiallyOwned: true, @"Local\ClaudeWidget.SingleInstance", out var isFirstInstance);
if (!isFirstInstance)
{
    return;
}

// WPF needs a single-threaded apartment, which top-level statements can't request directly.
var uiThread = new Thread(() => new WidgetApplication().Run()) { Name = "UI" };
uiThread.SetApartmentState(ApartmentState.STA);
uiThread.Start();
uiThread.Join();

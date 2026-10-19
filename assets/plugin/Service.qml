import QtQuick
import Quickshell
import Quickshell.Io
import "." as Local

Item {
  id: root
  Component.onCompleted: Local.ServiceBridge.service = root
  Component.onDestruction: {
    if (Local.ServiceBridge.service === root) Local.ServiceBridge.service = null
  }

  property QtObject shell: null
  property string backend: "libvirt"
  property string status: "unknown"
  property bool metricsAvailable: false
  property double sampledAt: 0
  property string metricsError: ""
  property real cpuPercent: 0
  property real memoryPercent: 0
  property string memoryUsage: "—"
  property string networkUsage: "—"
  property int processCount: 0
  property int uptimeSeconds: 0
  property string bootProfile: "unknown"
  property bool busy: false
  property string actionMessage: ""
  property string lastError: ""
  property string storageError: ""

  readonly property bool running: root.status === "running"
  readonly property string errorMessage: root.storageError || root.lastError || root.metricsError
  readonly property bool hasError: root.errorMessage !== ""
  readonly property string uptimeLabel: {
    if (!root.metricsAvailable) return "—"
    var seconds = Math.max(0, root.uptimeSeconds)
    var hours = Math.floor(seconds / 3600)
    var minutes = Math.floor((seconds % 3600) / 60)
    if (hours > 0) return hours + "h " + minutes + "m"
    return minutes + "m"
  }

  function percentValue(value) {
    var number = parseFloat(String(value || "0").replace("%", ""))
    return isFinite(number) ? Math.max(0, number) : 0
  }

  function applyMetrics(raw) {
    try {
      var data = JSON.parse(String(raw || "{}"))
      root.backend = String(data.backend || "libvirt")
      root.status = String(data.status || "unknown")
      root.metricsAvailable = data.available === true
      root.sampledAt = Number(data.sampledAt || 0)
      root.metricsError = root.running && !root.metricsAvailable ? "Windows statistics unavailable" : ""
      root.cpuPercent = root.percentValue(data.cpu)
      root.memoryPercent = root.percentValue(data.memoryPercent)
      root.memoryUsage = String(data.memory || "—")
      root.networkUsage = String(data.network || "—")
      root.processCount = Math.max(0, parseInt(String(data.pids || "0"), 10) || 0)
      root.uptimeSeconds = Math.max(0, parseInt(String(data.uptimeSeconds || "0"), 10) || 0)
      root.bootProfile = String(data.bootProfile || "unknown")
      root.storageError = data.storage ? String(data.storage.error || "") : "Could not check Windows SSD"
    } catch (error) {
      root.invalidateMetrics("Could not read VM usage")
    }
  }

  function invalidateMetrics(message) {
    root.metricsAvailable = false
    root.status = "unknown"
    root.memoryUsage = "—"
    root.networkUsage = "—"
    root.metricsError = message
    root.storageError = ""
  }

  function refresh() {
    if (!metricsProcess.running) metricsProcess.running = true
  }

  function openConsole() {
    Quickshell.execDetached(["ws", "console"])
  }

  function start() {
    if (root.busy) return
    root.busy = true
    root.actionMessage = "Starting"
    startProcess.running = true
  }

  function stop() {
    if (root.busy || !root.running) return
    root.busy = true
    root.actionMessage = "Stopping"
    stopProcess.running = true
  }

  function forceStop() {
    if (root.busy || !root.running) return
    root.busy = true
    root.actionMessage = "Force stopping"
    forceStopProcess.running = true
  }

  function reboot(profile) {
    if (root.busy || root.running) return
    root.busy = true
    root.actionMessage = "Rebooting"
    rebootProcess.command = ["ws", "reboot", profile]
    rebootProcess.running = true
  }

  function actionFinished(exitCode, failureMessage, details) {
    root.busy = false
    root.actionMessage = ""
    var lines = String(details || "").trim().split("\n")
    var reason = lines[lines.length - 1].replace(/^window-side:\s*/, "").trim()
    root.lastError = exitCode === 0 ? "" : (reason || failureMessage).slice(0, 400)
    root.refresh()
  }

  IpcHandler {
    target: "noel.window-side"

    function refresh(): string { root.refresh(); return "ok" }
    function status(): string { return root.status }
    function error(): string { return root.errorMessage }
    function openConsole(): void { root.openConsole() }
    function stop(): void { root.stop() }
  }

  Process {
    id: metricsProcess
    command: ["ws", "metrics"]
    stdout: StdioCollector {
      waitForEnd: true
      onStreamFinished: root.applyMetrics(text)
    }
    onExited: function(exitCode) {
      if (exitCode !== 0) root.invalidateMetrics("Could not check VM status")
    }
  }

  Process {
    id: startProcess
    command: ["ws", "launch"]
    stderr: StdioCollector { id: startError; waitForEnd: true }
    onExited: function(exitCode) { root.actionFinished(exitCode, "Windows did not start", startError.text) }
  }

  Process {
    id: rebootProcess
    command: ["ws", "reboot", "normal"]
    stderr: StdioCollector { id: rebootError; waitForEnd: true }
    onExited: function(exitCode) { root.actionFinished(exitCode, "Could not reboot into the selected profile", rebootError.text) }
  }

  Process {
    id: stopProcess
    command: ["ws", "stop"]
    stderr: StdioCollector { id: stopError; waitForEnd: true }
    onExited: function(exitCode) { root.actionFinished(exitCode, "Clean shutdown failed", stopError.text) }
  }

  Process {
    id: forceStopProcess
    command: ["ws", "force-stop"]
    stderr: StdioCollector { id: forceStopError; waitForEnd: true }
    onExited: function(exitCode) { root.actionFinished(exitCode, "Force stop failed", forceStopError.text) }
  }

  Timer {
    interval: root.running ? 5000 : 10000
    repeat: true
    running: true
    triggeredOnStart: true
    onTriggered: root.refresh()
  }
}

import QtQuick
import Quickshell
import Quickshell.Io
import "plugins/noel.window-side" as WindowSide

ShellRoot {
  id: fixture
  QtObject { id: host }
  QtObject {
    id: bar
    property var shell: host
    property bool vertical: false
    property int barSize: 35
    property string position: "top"
    property string fontFamily: "sans-serif"
    property color foreground: "#eeeeee"
    property color barForeground: foreground
    property color background: "#181c24"
    property color accent: "#c58b61"
    property color urgent: "#c58b61"
    property bool hasBorder: false
    property bool hovered: false
    property var activePopout: null
    function registerClickTarget(target) {}
    function unregisterClickTarget(target) {}
    function showTooltip(text, item) {}
    function hideTooltip(item) {}
    function requestPopout(owner) { activePopout = owner }
    function releasePopout(owner) { activePopout = null }
    function targetBelongsToWindow(target, window) { return true }
  }
  Loader { id: backend; sourceComponent: WindowSide.Service {} }
  PanelWindow {
    implicitWidth: 1280
    implicitHeight: 35
    anchors { top: true; left: true; right: true }
    color: "#181c24"
    WindowSide.BarWidget { id: first; bar: bar; x: 600 }
  }
  WindowSide.BarWidget { id: second; bar: bar }
  IpcHandler {
    target: "window-side-fixture"
    function state(): string {
      return JSON.stringify({available: backend.item.metricsAvailable,
        cpu: backend.item.cpuPercent, memory: backend.item.memoryUsage,
        uptime: backend.item.uptimeLabel, status: backend.item.status,
        busy: backend.item.busy, error: backend.item.lastError,
        errorMessage: backend.item.errorMessage, barError: first.hasError,
        shared: first.windowSideService === backend.item && second.windowSideService === backend.item})
    }
    function open(): void { first.open() }
    function apply(raw: string): void { backend.item.applyMetrics(raw) }
    function fail(): void { backend.item.invalidateMetrics("Offline") }
    function actionError(): void { backend.item.lastError = "Update failed" }
    function start(): void { backend.item.start() }
  }
}

import QtQuick
import Quickshell
import qs.Commons
import qs.Ui
import "." as Local

BarWidget {
  id: root
  moduleName: "noel.window-side"

  readonly property QtObject windowSideService: Local.ServiceBridge.service
  readonly property bool running: windowSideService ? windowSideService.running : false
  readonly property bool hasError: windowSideService ? windowSideService.hasError : false
  readonly property bool opened: panelLoader.item ? panelLoader.item.opened === true : false

  function injectPanel() {
    if (!panelLoader.item) return
    panelLoader.item.bar = root.bar
    panelLoader.item.service = root.windowSideService
    panelLoader.item.anchorItem = button
    panelLoader.item.hostWidget = root
  }

  function open() { if (panelLoader.item) panelLoader.item.open() }
  function close() { if (panelLoader.item) panelLoader.item.close() }
  function toggle() { if (panelLoader.item) panelLoader.item.toggle() }
  function closeForPopoutSwitch() {
    if (panelLoader.item) panelLoader.item.closeForPopoutSwitch()
  }

  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight

  Loader {
    id: panelLoader
    active: true
    visible: false
    source: Qt.resolvedUrl("Panel.qml")
    onLoaded: {
      root.injectPanel()
      Qt.callLater(root.injectPanel)
    }
  }

  WidgetButton {
    id: button
    anchors.fill: parent
    bar: root.bar
    text: root.hasError ? " !" : root.running && root.windowSideService && root.windowSideService.metricsAvailable
      ? " " + Math.round(root.windowSideService.cpuPercent) + "%"
      : ""
    foreground: root.hasError ? (bar ? bar.urgent : Color.urgent)
      : root.running ? Color.accent : Qt.darker(bar ? bar.barForeground : Color.foreground, 1.55)
    activeColor: root.hasError ? (bar ? bar.urgent : Color.urgent) : Color.accent
    active: root.opened
    useActiveColor: root.running
    labelVisible: true
    tooltipText: root.hasError ? "window-side · " + root.windowSideService.errorMessage : root.running && root.windowSideService
      ? (root.windowSideService.metricsAvailable ? "window-side · " + Math.round(root.windowSideService.cpuPercent) + "% cpu · "
        + root.windowSideService.memoryUsage : "window-side · Statistics unavailable")
      : root.windowSideService && root.windowSideService.status === "unknown" ? "window-side · Status unavailable" : "window-side · Stopped"
    onPressed: function(mouseButton) {
      if (mouseButton === Qt.MiddleButton && root.windowSideService)
        root.windowSideService.openConsole()
      else if (mouseButton === Qt.LeftButton)
        root.toggle()
    }
  }

  onBarChanged: injectPanel()
}

import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Commons
import qs.Ui

Panel {
  id: root
  moduleName: "noel.window-side"
  ipcTarget: "noel.window-side.panel"
  manageIpc: false

  property QtObject service: null
  property var anchorItem: null
  property var hostWidget: null
  property bool forceArmed: false

  readonly property var barIdentity: hostWidget || root
  readonly property color foreground: bar ? bar.foreground : Color.foreground
  // Keep metric cards readable even when the active popup theme is light.
  readonly property color cardBackground: "#0b111b"
  readonly property color cardForeground: "#d7dde5"
  readonly property color urgent: bar ? bar.urgent : Color.urgent
  readonly property string fontFamily: bar ? bar.fontFamily : Style.font.family
  readonly property string bootProfile: service ? service.bootProfile : "unknown"
  readonly property string statusLabel: !service ? "Checking"
    : service.busy ? service.actionMessage
    : service.hasError ? "Error"
    : service.status === "unknown" ? "Unknown"
    : service.running ? "Running" : "Stopped"
  readonly property color statusColor: !service || service.busy ? foreground
    : service.hasError ? urgent
    : service.status === "unknown" ? urgent
    : service.running ? Color.accent : Qt.darker(foreground, 1.8)
  // A profile waiting for its confirming second click.
  property string rebootArmed: ""
  readonly property var bootOptions: [
    { value: "normal", icon: "󰌢",
      label: rebootArmed === "normal" ? "Reboot now?" : "Normal",
      tooltip: bootProfile === "normal"
        ? "Current boot: linux-omarchy.\nLinux owns the RTX; Windows uses a virtual display."
        : "Reboot into linux-omarchy.\nLinux gets the RTX back; Windows uses a virtual display." },
    { value: "gpu", icon: "󰢮",
      label: rebootArmed === "gpu" ? "Reboot now?" : "RTX passthrough",
      tooltip: bootProfile === "gpu"
        ? "Current boot: linux-omarchy with RTX passthrough.\nWindows owns the RTX, shown through Looking Glass.\nLinux runs on Intel graphics."
        : "Reboot into linux-omarchy with RTX passthrough.\nWindows gets the RTX, shown through Looking Glass.\nLinux uses Intel graphics until you switch back." }
  ]

  function open() { root.controller.show() }
  function close() { root.controller.hide(); root.forceArmed = false }
  function toggle() { root.opened ? root.close() : root.open() }
  function closeForPopoutSwitch() { root.close() }
  function switchPanel(direction) {
    if (root.bar && typeof root.bar.switchPanelFrom === "function")
      return root.bar.switchPanelFrom(root.barIdentity, direction)
    return false
  }
  function chooseBoot(profile) {
    if (!root.service || profile === root.bootProfile) return
    if (root.service.running) {
      root.service.lastError = "Stop Windows before switching the boot"
      return
    }
    if (root.rebootArmed !== profile) {
      root.rebootArmed = profile
      rebootReset.restart()
      return
    }
    root.rebootArmed = ""
    root.service.reboot(profile)
  }

  function armForceStop() {
    if (!root.forceArmed) {
      root.forceArmed = true
      forceReset.restart()
      return
    }
    root.forceArmed = false
    if (root.service) root.service.forceStop()
  }

  onOpenedChanged: {
    if (opened && service) service.refresh()
    if (!opened) { forceArmed = false; rebootArmed = "" }
  }

  Timer {
    id: rebootReset
    interval: 6000
    repeat: false
    onTriggered: root.rebootArmed = ""
  }

  Timer {
    id: forceReset
    interval: 6000
    repeat: false
    onTriggered: root.forceArmed = false
  }

  KeyboardPanel {
    id: panel
    anchorItem: root.anchorItem
    owner: root.barIdentity
    bar: root.bar
    open: root.opened
    focusTarget: keyCatcher
    contentWidth: panel.fittedContentWidth(Style.space(360))
    contentHeight: panel.fittedContentHeight(contentColumn.implicitHeight)

    PanelKeyCatcher {
      id: keyCatcher
      anchors.fill: parent
      onCloseRequested: root.close()
      onTabRequested: function(direction) { root.switchPanel(direction) }

      Column {
        id: contentColumn
        width: parent.width
        spacing: Style.space(10)

        PanelHero {
          width: parent.width
          title: "window-side"
          foreground: root.foreground
          fontFamily: root.fontFamily
          iconComponent: Component {
            Text {
              text: ""
              color: root.service && root.service.running ? Color.accent : root.foreground
              font.family: root.fontFamily
              font.pixelSize: Style.font.display
            }
          }
          trailingControl: Component {
            Row {
              spacing: Style.space(6)

              Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: Style.space(8)
                height: width
                radius: width / 2
                color: root.statusColor
              }
              Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.statusLabel
                color: root.foreground
                font.family: root.fontFamily
                font.pixelSize: Style.font.caption
              }
            }
          }
        }

        PanelSeparator { foreground: root.foreground }

        GridLayout {
          width: parent.width
          columns: 2
          rowSpacing: Style.space(8)
          columnSpacing: Style.space(8)

          Repeater {
            model: root.service ? [
              { label: "Cpu", value: root.service.metricsAvailable ? Math.round(root.service.cpuPercent) + "%" : "—" },
              { label: "Memory", value: root.service.memoryUsage },
              { label: "Network total", value: root.service.networkUsage },
              { label: "Windows uptime", value: root.service.uptimeLabel }
            ] : []

            delegate: BorderSurface {
              required property var modelData
              Layout.fillWidth: true
              implicitHeight: Style.space(58)
              radius: Style.cornerRadius
              color: root.cardBackground
              borderSpec: Border.controlSpec("normal", root.cardForeground, Color.accent)

              Column {
                anchors.centerIn: parent
                spacing: Style.space(3)

                Text {
                  anchors.horizontalCenter: parent.horizontalCenter
                  text: modelData.label
                  color: root.cardForeground
                  opacity: 0.65
                  font.family: root.fontFamily
                  font.pixelSize: Style.font.caption
                }
                Text {
                  anchors.horizontalCenter: parent.horizontalCenter
                  horizontalAlignment: Text.AlignHCenter
                  text: modelData.value
                  color: root.cardForeground
                  font.family: root.fontFamily
                  font.pixelSize: Style.font.body
                  font.bold: true
                }
              }
            }
          }
        }

        Text {
          visible: root.service ? root.service.hasError : false
          width: parent.width
          text: root.service ? root.service.errorMessage : ""
          color: root.urgent
          font.family: root.fontFamily
          font.pixelSize: Style.font.bodySmall
          wrapMode: Text.WordWrap
        }

        PanelSeparator { foreground: root.foreground }

        ButtonGroup {
          anchors.horizontalCenter: parent.horizontalCenter
          options: root.bootOptions
          value: root.bootProfile
          enabled: root.service ? !root.service.busy : false
          focusable: false
          foreground: root.foreground
          fontFamily: root.fontFamily
          fontSize: Style.font.bodySmall
          onChanged: function(value) { root.chooseBoot(value) }
        }

        RowLayout {
          width: parent.width
          spacing: Style.space(8)

          Button {
            Layout.fillWidth: true
            text: root.service && root.service.running ? "Open" : "Start"
            iconText: root.service && root.service.running ? "󰍹" : "󰐥"
            enabled: root.service ? !root.service.busy && root.service.status !== "unknown"
              && (root.service.running || root.service.storageError === "") : false
            foreground: root.foreground
            fontFamily: root.fontFamily
            onClicked: {
              if (!root.service) return
              if (root.service.running) root.service.openConsole()
              else root.service.start()
            }
          }

          Button {
            Layout.fillWidth: true
            visible: root.service ? root.service.running && !root.service.busy : false
            text: "Stop"
            iconText: "󰓛"
            enabled: root.service ? root.service.running && !root.service.busy : false
            foreground: root.foreground
            fontFamily: root.fontFamily
            onClicked: if (root.service) root.service.stop()
          }
        }

        Button {
          width: parent.width
          visible: root.service ? root.service.running && !root.service.busy : false
          text: root.forceArmed ? "Confirm force stop" : "Force stop"
          iconText: root.forceArmed ? "󰜺" : "󰆴"
          enabled: root.service ? !root.service.busy && root.service.status !== "unknown" : false
          foreground: root.forceArmed ? root.urgent : root.foreground
          fontFamily: root.fontFamily
          onClicked: root.armForceStop()
        }
      }
    }
  }
}

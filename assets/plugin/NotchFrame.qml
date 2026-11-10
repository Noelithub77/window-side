import QtQuick
import qs.Commons
import "../noel.notch.bar/ui" as Notch

Notch.ShibumiPanel {
  id: frame
  required property Item controller
  required property Component body

  anchorItem: controller.anchorItem
  owner: controller.barIdentity
  // The noshell panel needs the real bar for its hover rules.
  bar: controller.bar.noshellHost
  open: controller.opened
  focusTarget: content.item
  contentWidth: frame.fittedContentWidth(Style.space(360))
  contentHeight: frame.fittedContentHeight(content.item ? content.item.contentHeight : 0)

  Loader {
    id: content
    anchors.fill: parent
    sourceComponent: frame.body
  }
}

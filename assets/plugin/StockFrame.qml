import QtQuick
import qs.Commons
import qs.Ui

KeyboardPanel {
  id: frame
  required property Item controller
  required property Component body

  anchorItem: controller.anchorItem
  owner: controller.barIdentity
  bar: controller.bar
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

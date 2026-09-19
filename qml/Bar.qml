import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

Window {
    id: root
    width: Screen.width
    height: 36
    visible: true
    color: "transparent"
    flags: Qt.FramelessWindowHint
    title: "Knave Shell Bar"

    Rectangle {
        anchors.fill: parent
        color: "#e6121923"
        border.color: "#26ffffff"
        border.width: 1

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 12
            anchors.rightMargin: 12
            spacing: 10

            ToolButton {
                id: overviewButton
                implicitWidth: 25
                implicitHeight: 25
                hoverEnabled: true
                onClicked: Shell.openOverview()
                Accessible.name: qsTr("Open workspace overview")

                background: Rectangle {
                    radius: 7
                    color: overviewButton.hovered ? "#24ffffff" : "transparent"
                }

                contentItem: Item {
                    Repeater {
                        model: 4
                        Rectangle {
                            required property int index
                            width: 5
                            height: 5
                            radius: 1.5
                            color: "#eaf3ff"
                            x: index % 2 * 8 + 4
                            y: Math.floor(index / 2) * 8 + 4
                        }
                    }
                }
            }

            Text {
                text: "Knave Shell"
                color: "#f4f7fb"
                font.pixelSize: 13
                font.weight: Font.DemiBold
            }

            Row {
                spacing: 3
                Repeater {
                    model: Shell.workspaces
                    Rectangle {
                        required property var modelData
                        width: modelData.active ? 31 : 25
                        height: 22
                        radius: 11
                        color: modelData.active ? "#3f52677f" : "transparent"
                        border.color: modelData.active ? "#30ffffff" : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: modelData.number
                            color: modelData.active ? "#ffffff" : "#aab6c6"
                            font.pixelSize: 11
                            font.weight: modelData.active ? Font.DemiBold : Font.Normal
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: Shell.focusWorkspace(modelData.number)
                        }
                    }
                }
            }

            Item { Layout.fillWidth: true }

            Text {
                id: clock
                color: "#e9eef5"
                font.pixelSize: 12
                horizontalAlignment: Text.AlignHCenter

                function update() {
                    text = Qt.formatDateTime(new Date(), "ddd MMM d   h:mm AP")
                }

                Component.onCompleted: update()
                Timer {
                    interval: 1000
                    running: true
                    repeat: true
                    onTriggered: clock.update()
                }
            }

            Item { Layout.fillWidth: true }

            Text {
                text: Shell.connected ? qsTr("Villain") : qsTr("Villain unavailable")
                color: Shell.connected ? "#91d8b4" : "#ffadad"
                font.pixelSize: 11
            }
        }
    }
}

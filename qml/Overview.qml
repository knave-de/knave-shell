import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

pragma ComponentBehavior: Bound

Window {
    id: root
    width: Screen.width
    height: Screen.height
    visible: true
    color: "#dc0d141d"
    flags: Qt.FramelessWindowHint
    title: "Knave Workspace Overview"

    property color textPrimary: "#f4f7fb"
    property color textSecondary: "#aeb9c8"
    property color panel: "#b8232d3a"
    property color accent: "#8dbbff"

    function rebuildResults() {
        const query = searchField.text.trim().toLowerCase()
        const results = []
        if (query.length === 0) {
            searchResults.model = results
            return
        }
        for (let index = 0; index < Shell.windows.length; ++index) {
            const window = Shell.windows[index]
            const haystack = (window.title + " " + window.appId + " workspace "
                              + window.workspace).toLowerCase()
            if (haystack.indexOf(query) !== -1) {
                results.push({
                    kind: "window",
                    id: window.id,
                    title: window.title,
                    detail: (window.appId || qsTr("Application")) + " · "
                            + qsTr("Workspace %1").arg(window.workspace)
                })
            }
        }
        for (let index = 0; index < Shell.workspaces.length; ++index) {
            const number = Shell.workspaces[index].number
            const title = qsTr("Workspace %1").arg(number)
            if (title.toLowerCase().indexOf(query) !== -1)
                results.push({kind: "workspace", workspace: number, title: title,
                              detail: qsTr("Switch workspace")})
        }
        if (qsTr("Close overview").toLowerCase().indexOf(query) !== -1
                || "escape".indexOf(query) !== -1) {
            results.push({kind: "close", title: qsTr("Close overview"),
                          detail: qsTr("Return to the desktop")})
        }
        searchResults.model = results
    }

    function activate(result) {
        if (!result)
            return
        if (result.kind === "window")
            Shell.focusWindow(result.id)
        else if (result.kind === "workspace")
            Shell.focusWorkspace(result.workspace)
        Shell.closeOverview()
    }

    Shortcut {
        sequence: "Escape"
        onActivated: Shell.closeOverview()
    }

    Rectangle {
        anchors.fill: parent
        color: "transparent"

        RowLayout {
            id: topBar
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            height: 38
            anchors.leftMargin: 15
            anchors.rightMargin: 15
            spacing: 14

            Text {
                text: "Knave Shell"
                color: root.textPrimary
                font.pixelSize: 13
                font.weight: Font.DemiBold
            }

            Row {
                spacing: 4
                Repeater {
                    model: Shell.workspaces
                    Rectangle {
                        required property var modelData
                        width: 34
                        height: 24
                        radius: 12
                        color: modelData.active ? "#40566e89" : "#13000000"

                        Text {
                            anchors.centerIn: parent
                            text: modelData.number
                            color: modelData.active ? root.textPrimary : root.textSecondary
                            font.pixelSize: 11
                        }
                    }
                }
            }

            Item { Layout.fillWidth: true }
            Text {
                id: overviewClock
                color: root.textPrimary
                font.pixelSize: 12
                function update() { text = Qt.formatDateTime(new Date(), "ddd MMM d   h:mm AP") }
                Component.onCompleted: update()
                Timer { interval: 1000; running: true; repeat: true; onTriggered: overviewClock.update() }
            }
            Item { Layout.fillWidth: true }
            Text {
                text: Shell.connected ? qsTr("Villain connected") : qsTr("Villain unavailable")
                color: Shell.connected ? "#91d8b4" : "#ffadad"
                font.pixelSize: 11
            }
        }

        Rectangle {
            id: searchBox
            anchors.top: topBar.bottom
            anchors.topMargin: Math.max(14, root.height * 0.025)
            anchors.horizontalCenter: parent.horizontalCenter
            width: Math.min(690, root.width * 0.52)
            height: 50
            radius: 25
            color: "#c6313c49"
            border.color: searchField.activeFocus ? "#669bc7ff" : "#28ffffff"
            border.width: 1

            Text {
                anchors.left: parent.left
                anchors.leftMargin: 19
                anchors.verticalCenter: parent.verticalCenter
                text: "⌕"
                color: root.textSecondary
                font.pixelSize: 24
            }

            TextField {
                id: searchField
                anchors.left: parent.left
                anchors.leftMargin: 53
                anchors.right: shortcutHint.left
                anchors.rightMargin: 12
                anchors.verticalCenter: parent.verticalCenter
                placeholderText: qsTr("Search windows and actions…")
                color: root.textPrimary
                placeholderTextColor: "#91a0b2"
                font.pixelSize: 14
                background: null
                focus: true
                onTextChanged: root.rebuildResults()
                onAccepted: {
                    if (searchResults.count > 0)
                        root.activate(searchResults.model[searchResults.currentIndex])
                }
                Keys.onDownPressed: searchResults.incrementCurrentIndex()
                Keys.onUpPressed: searchResults.decrementCurrentIndex()
                Component.onCompleted: forceActiveFocus()
            }

            Rectangle {
                id: shortcutHint
                anchors.right: parent.right
                anchors.rightMargin: 13
                anchors.verticalCenter: parent.verticalCenter
                width: 34
                height: 25
                radius: 6
                color: "#3c111820"
                Text {
                    anchors.centerIn: parent
                    text: "Esc"
                    color: root.textSecondary
                    font.pixelSize: 10
                }
            }
        }

        Rectangle {
            id: resultsPanel
            visible: searchField.text.trim().length > 0
            anchors.top: searchBox.bottom
            anchors.topMargin: 8
            anchors.horizontalCenter: searchBox.horizontalCenter
            width: searchBox.width
            height: Math.min(360, searchResults.contentHeight + 14)
            radius: 18
            color: "#f01a2430"
            border.color: "#26ffffff"
            z: 20

            ListView {
                id: searchResults
                anchors.fill: parent
                anchors.margins: 7
                clip: true
                spacing: 3
                currentIndex: 0

                delegate: Rectangle {
                    required property var modelData
                    required property int index
                    width: searchResults.width
                    height: 54
                    radius: 11
                    color: ListView.isCurrentItem ? "#395a7695" : "transparent"

                    Column {
                        anchors.left: parent.left
                        anchors.leftMargin: 14
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 2
                        Text { text: modelData.title; color: root.textPrimary; font.pixelSize: 13 }
                        Text { text: modelData.detail; color: root.textSecondary; font.pixelSize: 11 }
                    }
                    MouseArea {
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onEntered: searchResults.currentIndex = index
                        onClicked: root.activate(modelData)
                    }
                }
            }
        }

        ListView {
            id: workspaceList
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: searchBox.bottom
            anchors.bottom: footer.top
            anchors.topMargin: 35
            anchors.bottomMargin: 22
            orientation: ListView.Horizontal
            spacing: Math.max(18, root.width * 0.018)
            leftMargin: Math.max(30, root.width * 0.08)
            rightMargin: leftMargin
            clip: false
            header: Item { width: Math.max(30, root.width * 0.25); height: 1 }
            footer: Item { width: Math.max(30, root.width * 0.25); height: 1 }
            model: Shell.workspaces
            currentIndex: Math.max(0, Shell.activeWorkspace - 1)
            preferredHighlightBegin: width * 0.25
            preferredHighlightEnd: width * 0.75
            highlightRangeMode: ListView.ApplyRange

            onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Center)
            Component.onCompleted: positionViewAtIndex(currentIndex, ListView.Center)

            delegate: Item {
                id: workspaceDelegate
                required property var modelData
                required property int index
                width: modelData.active ? Math.min(680, root.width * 0.5)
                                        : Math.min(330, root.width * 0.235)
                height: workspaceList.height

                Column {
                    anchors.centerIn: parent
                    spacing: 13

                    Rectangle {
                        width: workspaceDelegate.width
                        height: width * 9 / 16
                        radius: modelData.active ? 22 : 14
                        color: "#192431"
                        border.width: modelData.active ? 3 : 1
                        border.color: modelData.active ? root.accent : "#40ffffff"
                        clip: true

                        Image {
                            anchors.fill: parent
                            anchors.margins: modelData.active ? 3 : 1
                            source: "image://workspace/" + modelData.number + "/" + Shell.previewRevision
                            fillMode: Image.PreserveAspectCrop
                            asynchronous: false
                            cache: false
                        }

                        Rectangle {
                            anchors.fill: parent
                            color: modelData.windowCount === 0 ? "#4d101823" : "transparent"
                            Text {
                                visible: modelData.windowCount === 0
                                anchors.centerIn: parent
                                text: qsTr("Empty workspace")
                                color: root.textSecondary
                                font.pixelSize: 13
                            }
                        }

                        Rectangle {
                            visible: modelData.active
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.bottom: parent.bottom
                            height: 45
                            gradient: Gradient {
                                GradientStop { position: 0; color: "transparent" }
                                GradientStop { position: 1; color: "#b8000000" }
                            }
                            Text {
                                anchors.left: parent.left
                                anchors.leftMargin: 15
                                anchors.bottom: parent.bottom
                                anchors.bottomMargin: 9
                                text: qsTr("%1 window(s)").arg(modelData.windowCount)
                                color: root.textPrimary
                                font.pixelSize: 11
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                Shell.focusWorkspace(modelData.number)
                                Shell.closeOverview()
                            }
                        }
                    }

                    Text {
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: qsTr("Workspace %1").arg(modelData.number)
                        color: root.textPrimary
                        font.pixelSize: modelData.active ? 15 : 13
                        font.weight: modelData.active ? Font.DemiBold : Font.Normal
                    }

                    Rectangle {
                        visible: modelData.active
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: 8
                        height: 8
                        radius: 4
                        color: root.accent
                    }
                }
            }
        }

        RowLayout {
            id: footer
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.leftMargin: 20
            anchors.rightMargin: 20
            anchors.bottomMargin: 14
            height: 42

            Column {
                spacing: 2
                Text { text: "KNAVE SHELL"; color: "#c8d1dc"; font.pixelSize: 9; font.letterSpacing: 4 }
                Text { text: "WORKSPACES AT A GLANCE"; color: "#718091"; font.pixelSize: 7; font.letterSpacing: 2 }
            }
            Item { Layout.fillWidth: true }
            Text {
                visible: !Shell.connected
                text: Shell.lastError
                color: "#ffb3b3"
                font.pixelSize: 11
                elide: Text.ElideRight
                Layout.maximumWidth: root.width * 0.45
            }
        }
    }
}

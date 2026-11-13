import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.kde.plasma.plasmoid

PlasmoidItem {
    implicitWidth: 240
    implicitHeight: 120

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 8

        Label {
            text: "Peerfora"
            font.bold: true
        }

        Label {
            text: "Development preview"
            wrapMode: Text.Wrap
        }
    }
}

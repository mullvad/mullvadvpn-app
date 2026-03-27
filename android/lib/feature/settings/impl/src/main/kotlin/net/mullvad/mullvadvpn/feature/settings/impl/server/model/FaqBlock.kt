package net.mullvad.mullvadvpn.feature.settings.impl.server.model

sealed class FaqBlock {
    data class Question(val title: String, val content: List<Content>) : FaqBlock()

    sealed class Content {
        data class Paragraph(val text: String) : Content()
        data class ListItem(val text: String) : Content()
    }
}

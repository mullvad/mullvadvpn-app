package net.mullvad.mullvadvpn.feature.settings.impl.server.model

data class RichText(
    val parts: List<Part>
) {
    sealed class Part {
        data class Text(val text: String) : Part()
        data class Link(val text: String, val url: String) : Part()
    }
}

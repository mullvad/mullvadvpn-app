package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlin.text.findAnyOf
import kotlin.text.isNotEmpty
import kotlin.text.substring
import kotlinx.parcelize.Parcelize

@Parcelize
data class HighlightedString(val highlights: List<IntRange>, val text: String) : Parcelable {
    init {
        require(highlights.all { it.first >= 0 && it.last < text.length }) {
            "Highlights must be within the bounds of the text"
        }
    }

    constructor(highlight: IntRange, text: String) : this(listOf(highlight), text)

    companion object {
        fun partialMatch(
            text: String,
            query: String,
            ignoreCase: Boolean = true,
            limit: Int = 0,
        ): HighlightedString {
            if (query.isEmpty()) return HighlightedString(emptyList(), text)
            val words = query.split(" ").filter { it.isNotBlank() }
            val highlights = mutableListOf<IntRange>()
            var remaining = text
            var offset = 0
            while (remaining.isNotEmpty()) {
                val (matchIndex, matchString) =
                    remaining.findAnyOf(words, ignoreCase = ignoreCase) ?: (-1 to "")
                if (matchIndex == -1 || (limit > 0 && highlights.size >= limit)) {
                    break
                }
                highlights.add(offset + matchIndex..<matchIndex + offset + matchString.length)
                remaining = remaining.substring(matchIndex + matchString.length)
                offset += matchIndex + matchString.length
            }
            return HighlightedString(highlights, text)
        }

        fun fromString(text: String): HighlightedString = HighlightedString(emptyList(), text)
    }
}

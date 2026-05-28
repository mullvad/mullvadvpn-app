package net.mullvad.mullvadvpn.lib.model

import arrow.core.NonEmptyList
import kotlin.collections.emptyList

data class HighlightedString(val highlights: NonEmptyList<IntRange>, val text: String) {
    init {
        require(highlights.all { it.first >= 0 && it.last < text.length }) {
            "Highlights must be within the bounds of the text"
        }
    }

    companion object {
        fun findHighlights(text: String, query: String): HighlightedString? {
            if (query.isEmpty()) return null

            val matchedRanges = findMatchRanges(text, query)
            return if (matchedRanges.isEmpty()) null
            else HighlightedString(NonEmptyList.of(matchedRanges), text)
        }

        private fun findMatchRanges(
            text: String,
            term: String,
            ignoreCase: Boolean = true,
        ): List<IntRange> {
            val ranges = mutableListOf<IntRange>()
            var startIndex = 0

            while (startIndex <= text.length - term.length) {
                val index = text.indexOf(term, startIndex = startIndex, ignoreCase = ignoreCase)
                if (index == -1) break

                ranges += index..<index + term.length
                startIndex = index + 1 // use + term.length if you want non-overlapping matches only
            }

            return ranges
        }

        fun fromString(text: String): HighlightedString =
            HighlightedString(NonEmptyList(IntRange(0, 0), emptyList()), text)
    }
}

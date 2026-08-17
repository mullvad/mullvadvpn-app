package net.mullvad.mullvadvpn.lib.model

@Suppress("ReturnCount")
fun String.search(query: String): SearchMatch? {
    val trimmedQuery = query.trim()
    if (trimmedQuery.isEmpty()) return null

    val matchIndex = indexOf(trimmedQuery, ignoreCase = true)

    // Substring match
    return if (matchIndex != -1) {
        val matchRange = matchIndex..<(matchIndex + trimmedQuery.length)

        // First word starts with
        if (matchIndex == 0) {
            return SearchMatch(text = this, score = 1000, matchRange = matchRange)
        }

        // Other word starts with
        split(" ").withIndex().forEach { (wordIndex, word) ->
            if (word.startsWith(trimmedQuery, ignoreCase = true)) {
                return SearchMatch(
                    text = this,
                    // earlier word = higher score
                    score = 1000 - wordIndex,
                    matchRange = matchRange,
                )
            }
        }

        // Contains match
        SearchMatch(text = this, score = 500 - matchIndex, matchRange = matchRange)
    } else {
        // Fuzzy match
        fuzzyMatch(trimmedQuery)?.let { fuzzyRange ->
            val firstCharIndex = fuzzyRange[0].first
            SearchMatch(text = this, score = 300 - firstCharIndex, matchRange = fuzzyRange)
        }
    }
}

private fun String.fuzzyMatch(query: String): List<IntRange>? = buildList {
    query.fold(0) { startIndex, char ->
        val found = indexOf(char, startIndex, ignoreCase = true)

        if (found == -1) return null

        add(found..found)
        found + 1
    }
}

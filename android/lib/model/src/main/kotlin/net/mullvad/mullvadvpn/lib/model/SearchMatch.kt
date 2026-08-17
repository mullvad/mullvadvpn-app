package net.mullvad.mullvadvpn.lib.model

data class SearchMatch(val text: String, val score: Int, val matchRange: List<IntRange>) :
    Comparable<SearchMatch> {

    constructor(
        text: String,
        score: Int,
        matchRange: IntRange,
    ) : this(text, score, listOf(matchRange))

    constructor(
        text: String,
        score: Int,
    ) : this(text, score, emptyList())

    override fun compareTo(other: SearchMatch): Int = this.score.compareTo(other.score)
}

data class RelayListSearchResult(
    /// The list of countries that either matched or had a matching city or relay for the query
    val matchedCountries: List<RelayItem.Location.Country>,
    /// The set of countries/cities that should be expanded by default in the UI
    val expansionSet: Set<RelayItemId>,
    /// Maps a matched item to a SearchMatch that is used to highlight the match in the UI
    val highlights: Map<RelayItemId, SearchMatch>,
)

data class CustomListSearchResult(
    /// The list of countries that either matched or had a matching city or relay for the query
    val matchedCustomLists: List<RelayItem.CustomList>,
    /// Maps a matched item to a SearchMatch that is used to highlight the match in the UI
    val highlights: Map<RelayItemId, SearchMatch>,
)

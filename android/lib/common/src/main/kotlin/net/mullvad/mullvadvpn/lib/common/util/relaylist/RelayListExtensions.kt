package net.mullvad.mullvadvpn.lib.common.util.relaylist

import java.text.Collator
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayListSearchResult
import net.mullvad.mullvadvpn.lib.model.SearchMatch
import net.mullvad.mullvadvpn.lib.model.search

fun List<RelayItem.Location.Country>.newFilterOnSearch(searchTerm: String): RelayListSearchResult {
    return SearchProcessor(searchTerm).process(this)
}

private class SearchProcessor(private val searchTerm: String) {
    val expansionSet = mutableSetOf<RelayItemId>()
    val highlights = mutableMapOf<RelayItemId, SearchMatch>()

    fun process(countries: List<RelayItem.Location.Country>): RelayListSearchResult {
        val matchedCountries =
            countries
                .mapNotNull { processCountry(it) }
                .sortedByDescending { it.second }
                .map { it.first }

        return RelayListSearchResult(
            matchedCountries = matchedCountries,
            expansionSet = expansionSet,
            highlights = highlights,
        )
    }

    private fun processCountry(
        country: RelayItem.Location.Country
    ): Pair<RelayItem.Location.Country, SearchMatch>? {

        val scoredCities =
            country.cities.mapNotNull { processCity(it) }.sortedByDescending { it.second }

        val countryMatch = country.name.search(searchTerm)

        return if (scoredCities.isEmpty()) {
            val match = countryMatch ?: return null

            highlights[country.id] = match
            country to match
        } else {
            expansionSet.add(country.id)

            if (countryMatch != null) {
                highlights[country.id] = countryMatch
            }

            // If a child's score is higher than its parent's, we need to update the parent's score
            // to the child's so that the sort order is correct.
            val maxChildScore = scoredCities[0].second.score
            val highestScore = maxOf(maxChildScore, countryMatch?.score ?: 0)

            // If we don't have a match on the country name, we still need to create a SearchMatch
            // so that the sorting works.
            val finalMatch =
                countryMatch?.copy(score = highestScore)
                    ?: SearchMatch(text = country.name, score = highestScore)

            val filteredCities = scoredCities.map { it.first }

            country.copy(cities = filteredCities) to finalMatch
        }
    }

    private fun processCity(
        city: RelayItem.Location.City
    ): Pair<RelayItem.Location.City, SearchMatch>? {

        val scoredRelays =
            city.relays
                .mapNotNull { relay ->
                    relay.name.search(searchTerm)?.let { match ->
                        highlights[relay.id] = match
                        relay to match
                    }
                }
                .sortedByDescending { it.second }

        val cityMatch = city.name.search(searchTerm)

        return if (scoredRelays.isEmpty()) {
            val match = cityMatch ?: return null

            highlights[city.id] = match
            city to match
        } else {
            expansionSet.add(city.id)

            if (cityMatch != null) {
                highlights[city.id] = cityMatch
            }

            val maxChildScore = scoredRelays[0].second.score
            val highestScore = maxOf(maxChildScore, cityMatch?.score ?: 0)

            val finalMatch =
                cityMatch?.copy(score = highestScore)
                    ?: SearchMatch(text = city.name, score = highestScore)

            val filteredRelays = scoredRelays.map { it.first }

            city.copy(relays = filteredRelays) to finalMatch
        }
    }
}

fun GeoLocationId.ancestors(): List<GeoLocationId> =
    when (this) {
        is GeoLocationId.City -> listOf(country)
        is GeoLocationId.Country -> emptyList()
        is GeoLocationId.Hostname -> listOf(country, city)
    }

fun List<RelayItem.Location.Country>.getRelayItemsByCodes(
    codes: List<GeoLocationId>
): List<RelayItem.Location> =
    this.filter { codes.contains(it.id) } +
        this.flatMap { it.descendants() }.filter { codes.contains(it.id) }

// Sort using the default locale's collation rules rather than raw Unicode value comparison.
fun <T : RelayItem> List<T>.sortedByName() =
    this.sortedWith(compareBy(Collator.getInstance()) { it.name })

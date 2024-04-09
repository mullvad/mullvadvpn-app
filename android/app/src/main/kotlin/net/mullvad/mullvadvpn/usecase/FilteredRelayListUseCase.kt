package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository

class FilteredRelayListUseCase(
    private val relayListRepository: RelayListRepository,
    private val relayListFilterRepository: RelayListFilterRepository
) {
    fun filteredRelayList() =
        combine(
            relayListRepository.relayList,
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders
        ) { relayList, selectedOwnership, selectedProviders ->
            relayList.countries
                .filterOnOwnership(selectedOwnership)
                .filterOnProviders(selectedProviders)
        }

    private fun List<RelayItem.Location.Country>.filterOnOwnership(
        ownership: Constraint<Ownership>
    ): List<RelayItem.Location.Country> =
        when (ownership) {
            is Constraint.Only -> {
                val selectedOwnership = ownership.value
                this.filter { country ->
                    country.cities.any { city ->
                        city.relays.any { relay -> relay.ownership == selectedOwnership }
                    }
                }
            }
            else -> this
        }

    private fun List<RelayItem.Location.Country>.filterOnProviders(
        providers: Constraint<Providers>
    ): List<RelayItem.Location.Country> =
        when (providers) {
            is Constraint.Only -> {
                val selectedProviders = providers.value
                this.filter { country ->
                    country.cities.any { city ->
                        city.relays.any { relay ->
                            selectedProviders.providers.contains(relay.provider)
                        }
                    }
                }
            }
            else -> this
        }
}

package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.Provider
import net.mullvad.mullvadvpn.repository.RelayListRepository

class AvailableProvidersUseCase(private val relayListRepository: RelayListRepository) {

    fun availableProviders(): Flow<List<Provider>> =
        relayListRepository.relayList.map { relayList ->
            relayList.countries
                .flatMap(RelayItem.Location.Country::cities)
                .flatMap(RelayItem.Location.City::relays)
                .map { relay ->
                    Provider(relay.provider, relay.ownership == Ownership.MullvadOwned)
                }
                .distinct()
        }
}

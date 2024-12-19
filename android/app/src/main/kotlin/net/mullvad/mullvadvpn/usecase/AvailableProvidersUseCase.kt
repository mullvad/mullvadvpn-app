package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.repository.RelayListRepository

class AvailableProvidersUseCase(private val relayListRepository: RelayListRepository) {

    operator fun invoke(): Flow<List<Provider>> =
        relayListRepository.relayList.map { relayList ->
            relayList
                .flatMap(RelayItem.Location.Country::cities)
                .flatMap(RelayItem.Location.City::relays)
                .groupBy({ it.provider }, { it.ownership })
                .map { (provider, ownerships) -> Provider(provider, ownerships.toSet()) }
                .sortedBy { it.providerId.value.uppercase() }
        }
}

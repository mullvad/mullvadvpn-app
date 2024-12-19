package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.repository.RelayListRepository

class AvailableProvidersUseCase(private val relayListRepository: RelayListRepository) {
    operator fun invoke(): Flow<List<ProviderId>> =
        relayListRepository.relayList.map { relayList ->
            relayList
                .flatMap(RelayItem.Location.Country::cities)
                .flatMap(RelayItem.Location.City::relays)
                .map { it.provider }
                .distinct()
        }
}

class ProviderOwnershipUseCase(private val relayListRepository: RelayListRepository) {
    operator fun invoke(): Flow<Map<ProviderId, Set<Ownership>>> =
        relayListRepository.relayList.map { relayList ->
            relayList
                .flatMap(RelayItem.Location.Country::cities)
                .flatMap(RelayItem.Location.City::relays)
                .groupBy({ it.provider }, { it.ownership })
                .mapValues { (_, ownerships) -> ownerships.toSet() }
        }
}

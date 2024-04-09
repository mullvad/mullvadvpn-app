package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.repository.RelayListRepository

class PortRangeUseCase(private val relayListRepository: RelayListRepository) {
    fun portRanges(): Flow<List<PortRange>> =
        relayListRepository.wireguardEndpointData.map { it.portRanges }.distinctUntilChanged()
}

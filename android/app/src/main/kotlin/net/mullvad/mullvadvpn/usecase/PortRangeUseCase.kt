package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener

class PortRangeUseCase(private val relayListListener: RelayListListener) {
    fun portRanges(): Flow<List<PortRange>> =
        relayListListener.relayListEvents
            .map { it?.wireguardEndpointData?.portRanges ?: emptyList() }
            .distinctUntilChanged()
}

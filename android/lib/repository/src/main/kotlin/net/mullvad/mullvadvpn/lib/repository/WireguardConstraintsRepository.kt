package net.mullvad.mullvadvpn.lib.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.RelayItemId

class WireguardConstraintsRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val wireguardConstraints =
        managementService.settings
            .map { it.relaySettings.relayConstraints.wireguardConstraints }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)

    suspend fun setMultihop(multihopMode: MultihopMode) =
        managementService.setMultihop(multihopMode)

    suspend fun setEntryLocation(relayItemId: Constraint<RelayItemId>) =
        managementService.setEntryLocation(relayItemId)

    suspend fun setDeviceIpVersion(ipVersion: Constraint<IpVersion>) =
        managementService.setDeviceIpVersion(ipVersion)

    suspend fun setMultihopAndEntryLocation(
        multihopMode: MultihopMode,
        entryRelayItemId: Constraint<RelayItemId>,
    ) = managementService.setMultihopAndEntryLocation(multihopMode, entryRelayItemId)
}

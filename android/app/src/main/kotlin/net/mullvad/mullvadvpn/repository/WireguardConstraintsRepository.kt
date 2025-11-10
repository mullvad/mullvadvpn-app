package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.RelayItemId

class WireguardConstraintsRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val wireguardConstraints =
        managementService.settings
            .mapNotNull { it.relaySettings.relayConstraints.wireguardConstraints }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)

    suspend fun setWireguardPort(port: Constraint<Port>) = managementService.setWireguardPort(port)

    suspend fun setMultihop(enabled: Boolean) = managementService.setMultihop(enabled)

    suspend fun setEntryLocation(relayItemId: RelayItemId) =
        managementService.setEntryLocation(relayItemId)

    suspend fun setDeviceIpVersion(ipVersion: Constraint<IpVersion>) =
        managementService.setDeviceIpVersion(ipVersion)
}

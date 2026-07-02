package net.mullvad.mullvadvpn.lib.repository

import arrow.core.Either
import arrow.core.right
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.SetWireguardConstraintsError

class WireguardConstraintsRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val wireguardConstraints =
        managementService.settings
            .mapNotNull { it.relaySettings.relayConstraints.wireguardConstraints }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)

    suspend fun setMultihop(enabled: Boolean) = managementService.setMultihop(enabled)

    suspend fun setEntryLocation(relayItemId: Constraint<RelayItemId>) =
        managementService.setEntryLocation(relayItemId)

    suspend fun setDeviceIpVersion(ipVersion: Constraint<IpVersion>) =
        managementService.setDeviceIpVersion(ipVersion)

    suspend fun setMultihopAndEntryLocation(
        multihopEnabled: Boolean,
        entryRelayItemId: RelayItemId,
    ) = managementService.setMultihopAndEntryLocation(multihopEnabled, entryRelayItemId)

    @Suppress("UNUSED_PARAMETER")
    // TODO Remove and replace with actual implementation
    suspend fun setMultihopMode(
        multihopMode: MultihopMode
    ): Either<SetWireguardConstraintsError, Unit> {
        return Unit.right()
    }
}

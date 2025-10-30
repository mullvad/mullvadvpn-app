package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayListType

class RelayListFilterRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val selectedOwnership: StateFlow<Constraint<Ownership>> =
        managementService.settings
            .map { settings -> settings.relaySettings.relayConstraints.ownership }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    val selectedProviders: StateFlow<Constraint<Providers>> =
        managementService.settings
            .map { settings -> settings.relaySettings.relayConstraints.providers }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    val selectedMultihopEntryOwnership: StateFlow<Constraint<Ownership>> =
        managementService.settings
            .map { settings ->
                settings.relaySettings.relayConstraints.wireguardConstraints.entryOwnership
            }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    val selectedMultihopEntryProviders: StateFlow<Constraint<Providers>> =
        managementService.settings
            .map { settings ->
                settings.relaySettings.relayConstraints.wireguardConstraints.entryProviders
            }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    suspend fun updateSelectedOwnershipAndProviderFilter(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>,
        relayListType: RelayListType,
    ) = managementService.setOwnershipAndProviders(ownership, providers, relayListType)

    suspend fun updateSelectedOwnership(
        ownership: Constraint<Ownership>,
        relayListType: RelayListType,
    ) = managementService.setOwnership(ownership, relayListType)

    suspend fun updateSelectedProviders(
        providers: Constraint<Providers>,
        relayListType: RelayListType,
    ) = managementService.setProviders(providers, relayListType)
}

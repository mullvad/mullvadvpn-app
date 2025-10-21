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
import net.mullvad.mullvadvpn.lib.model.isMultihopEntry

class RelayListFilterRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    private val selectedOwnership: StateFlow<Constraint<Ownership>> =
        managementService.settings
            .map { settings -> settings.relaySettings.relayConstraints.ownership }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    private val selectedProviders: StateFlow<Constraint<Providers>> =
        managementService.settings
            .map { settings -> settings.relaySettings.relayConstraints.providers }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    private val selectedMultihopEntryOwnership: StateFlow<Constraint<Ownership>> =
        managementService.settings
            .map { settings ->
                settings.relaySettings.relayConstraints.wireguardConstraints.entryOwnership
            }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    private val selectedMultihopEntryProviders: StateFlow<Constraint<Providers>> =
        managementService.settings
            .map { settings ->
                settings.relaySettings.relayConstraints.wireguardConstraints.entryProviders
            }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    fun selectedOwnership(filterType: RelayListType): StateFlow<Constraint<Ownership>> =
        if (filterType.isMultihopEntry()) {
            selectedMultihopEntryOwnership
        } else {
            selectedOwnership
        }

    fun selectedProviders(filterType: RelayListType): StateFlow<Constraint<Providers>> =
        if (filterType.isMultihopEntry()) {
            selectedMultihopEntryProviders
        } else {
            selectedProviders
        }

    suspend fun updateSelectedOwnershipAndProviderFilter(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>,
        filterType: RelayListType,
    ) = managementService.setOwnershipAndProviders(ownership, providers, filterType)

    suspend fun updateSelectedOwnership(value: Constraint<Ownership>, filterType: RelayListType) =
        managementService.setOwnership(value, filterType)

    suspend fun updateSelectedProviders(value: Constraint<Providers>, filterType: RelayListType) =
        managementService.setProviders(value, filterType)
}

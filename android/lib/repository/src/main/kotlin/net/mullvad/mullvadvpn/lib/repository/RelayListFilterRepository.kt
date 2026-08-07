package net.mullvad.mullvadvpn.lib.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayHopType

class RelayListFilterRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val selectedEntryOwnership: StateFlow<Constraint<Ownership>> =
        managementService.settings
            .map { settings ->
                settings.relaySettings.relayConstraints.wireguardConstraints.entryOwnership
            }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    val selectedExitOwnership: StateFlow<Constraint<Ownership>> =
        managementService.settings
            .map { settings -> settings.relaySettings.relayConstraints.ownership }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    fun selectedOwnership(hopType: RelayHopType): StateFlow<Constraint<Ownership>> =
        when (hopType) {
            RelayHopType.ENTRY -> selectedEntryOwnership
            RelayHopType.EXIT -> selectedExitOwnership
        }

    val selectedEntryProviders: StateFlow<Constraint<Providers>> =
        managementService.settings
            .map { settings ->
                settings.relaySettings.relayConstraints.wireguardConstraints.entryProviders
            }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    val selectedExitProviders: StateFlow<Constraint<Providers>> =
        managementService.settings
            .map { settings -> settings.relaySettings.relayConstraints.providers }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    fun selectedProviders(hopType: RelayHopType): StateFlow<Constraint<Providers>> =
        when (hopType) {
            RelayHopType.ENTRY -> selectedEntryProviders
            RelayHopType.EXIT -> selectedExitProviders
        }

    fun hasAnyFilterFlow(): Flow<FilterActiveState> =
        combine(
            selectedEntryOwnership,
            selectedEntryProviders,
            selectedExitOwnership,
            selectedExitProviders,
        ) { entryOwnership, entryProviders, exitOwnership, exitProviders ->
            FilterActiveState(
                hasAnyEntryFilter =
                    entryOwnership != Constraint.Any || entryProviders != Constraint.Any,
                hasAnyExitFilter =
                    exitOwnership != Constraint.Any || exitProviders != Constraint.Any,
            )
        }

    suspend fun updateSelectedOwnershipAndProviderFilter(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>,
        filterTarget: RelayHopType,
    ) =
        managementService.setOwnershipAndProviders(
            ownershipConstraint = ownership,
            providersConstraint = providers,
            hopType = filterTarget,
        )

    suspend fun updateSelectedOwnership(value: Constraint<Ownership>, hopType: RelayHopType) =
        managementService.setOwnership(value, hopType)

    suspend fun updateSelectedProviders(value: Constraint<Providers>, hopType: RelayHopType) =
        managementService.setProviders(value, hopType)
}

data class FilterActiveState(val hasAnyEntryFilter: Boolean, val hasAnyExitFilter: Boolean)

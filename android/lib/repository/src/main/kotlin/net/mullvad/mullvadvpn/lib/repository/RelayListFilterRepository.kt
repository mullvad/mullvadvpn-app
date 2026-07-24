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
import net.mullvad.mullvadvpn.lib.model.FilterTarget
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers

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

    fun selectedOwnership(filterTarget: FilterTarget): StateFlow<Constraint<Ownership>> =
        when (filterTarget) {
            FilterTarget.Entry -> selectedEntryOwnership
            FilterTarget.Exit -> selectedExitOwnership
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

    fun selectedProviders(filterTarget: FilterTarget): StateFlow<Constraint<Providers>> =
        when (filterTarget) {
            FilterTarget.Entry -> selectedEntryProviders
            FilterTarget.Exit -> selectedExitProviders
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
        filterTarget: FilterTarget,
    ) =
        managementService.setOwnershipAndProviders(
            ownershipConstraint = ownership,
            providersConstraint = providers,
            filterTarget = filterTarget,
        )

    suspend fun updateSelectedOwnership(value: Constraint<Ownership>, filterTarget: FilterTarget) =
        managementService.setOwnership(value, filterTarget)

    suspend fun updateSelectedProviders(value: Constraint<Providers>, filterTarget: FilterTarget) =
        managementService.setProviders(value, filterTarget)
}

data class FilterActiveState(val hasAnyEntryFilter: Boolean, val hasAnyExitFilter: Boolean)

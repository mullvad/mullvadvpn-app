package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.getOrElse
import arrow.core.raise.either
import co.touchlab.kermit.Logger
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.state.FilterChip
import net.mullvad.mullvadvpn.compose.state.RelayListSelection
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState.Content
import net.mullvad.mullvadvpn.compose.state.toSelectedProviders
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.SelectedLocation
import net.mullvad.mullvadvpn.relaylist.descendants
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.AvailableProvidersUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase

@Suppress("TooManyFunctions")
class SelectLocationViewModel(
    private val relayListFilterRepository: RelayListFilterRepository,
    private val availableProvidersUseCase: AvailableProvidersUseCase,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    private val filteredCustomListRelayItemsUseCase: FilterCustomListsRelayItemUseCase,
    private val customListsRepository: CustomListsRepository,
    private val customListActionUseCase: CustomListActionUseCase,
    private val filteredRelayListUseCase: FilteredRelayListUseCase,
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val selectedLocationUseCase: SelectedLocationUseCase,
) : ViewModel() {
    private val _relayListSelection: MutableStateFlow<RelayListSelection> =
        MutableStateFlow(RelayListSelection.Entry)
    private val _expandedItems = MutableStateFlow(initialExpand())

    @Suppress("DestructuringDeclarationWithTooManyEntries")
    val uiState =
        combine(
                relayListItems(),
                filterChips(),
                customListsRelayItemUseCase(),
                wireguardConstraintsRepository.wireguardConstraints,
                _relayListSelection,
            ) { relayListItems, filterChips, customLists, wireguardConstraints, relayListSelection
                ->
                Content(
                    filterChips = filterChips,
                    relayListItems = relayListItems,
                    customLists = customLists,
                    multihopEnabled = wireguardConstraints?.useMultihop ?: false,
                    relayListSelection = relayListSelection,
                )
            }
            .stateIn(viewModelScope, SharingStarted.Lazily, SelectLocationUiState.Loading)

    private val _uiSideEffect = Channel<SelectLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun initialExpand(): Set<String> = buildSet {
        Logger.d("initialExpand: ${selectedLocationUseCase().value}")
        when (
            val item =
                selectedLocationUseCase().value.getForRelayListSelect(_relayListSelection.value)
        ) {
            is GeoLocationId.City -> {
                Logger.d("GC item: $item")
                add(item.country.code)
            }
            is GeoLocationId.Hostname -> {
                Logger.d("GH item: $item")
                add(item.country.code)
                add(item.city.code)
            }
            is CustomListId,
            is GeoLocationId.Country,
            null -> {
                Logger.d("NO item: $item")
                /* No expands */
            }
        }
    }

    private fun filterChips() =
        combine(
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders,
            availableProvidersUseCase(),
            settingsRepository.settingsUpdates,
        ) { selectedOwnership, selectedConstraintProviders, allProviders, settings ->
            val ownershipFilter = selectedOwnership.getOrNull()
            val providerCountFilter =
                when (selectedConstraintProviders) {
                    is Constraint.Any -> null
                    is Constraint.Only ->
                        filterSelectedProvidersByOwnership(
                                selectedConstraintProviders.toSelectedProviders(allProviders),
                                ownershipFilter,
                            )
                            .size
                }
            buildList {
                if (ownershipFilter != null) {
                    add(FilterChip.Ownership(ownershipFilter))
                }
                if (providerCountFilter != null) {
                    add(FilterChip.Provider(providerCountFilter))
                }
                if (settings?.isDaitaEnabled() == true) {
                    add(FilterChip.Daita)
                }
            }
        }

    private fun relayListItems() =
        combine(
            filteredRelayListUseCase(),
            filteredCustomListRelayItemsUseCase(),
            selectedLocationUseCase(),
            _expandedItems,
            _relayListSelection,
        ) { relayCountries, customLists, selectedItem, expandedItems, relayListSelection ->
            relayListItems(
                relayCountries = relayCountries,
                customLists = customLists,
                selectedItem = selectedItem.getForRelayListSelect(relayListSelection),
                expandedItems = expandedItems,
            )
        }

    private fun RelayItemId.expandKey(parent: CustomListId? = null) =
        (parent?.value ?: "") +
            when (this) {
                is CustomListId -> value
                is GeoLocationId -> code
            }

    fun selectRelay(relayItem: RelayItem) {
        viewModelScope.launch {
            val locationConstraint = relayItem.id
            when (_relayListSelection.value) {
                RelayListSelection.Entry -> selectEntryLocation(locationConstraint)
                RelayListSelection.Exit -> selectExitLocation(locationConstraint)
            }
        }
    }

    private suspend fun selectEntryLocation(locationConstraint: RelayItemId) {
        wireguardConstraintsRepository
            .setEntryLocation(locationConstraint)
            .fold(
                { _uiSideEffect.send(SelectLocationSideEffect.GenericError) },
                { _uiSideEffect.send(SelectLocationSideEffect.CloseScreen) },
            )
    }

    private suspend fun selectExitLocation(locationConstraint: RelayItemId) {
        relayListRepository
            .updateSelectedRelayLocation(locationConstraint)
            .fold(
                { _uiSideEffect.send(SelectLocationSideEffect.GenericError) },
                { _uiSideEffect.send(SelectLocationSideEffect.CloseScreen) },
            )
    }

    fun onToggleExpand(item: RelayItemId, parent: CustomListId? = null, expand: Boolean) {
        _expandedItems.update {
            val key = item.expandKey(parent)
            if (expand) {
                it + key
            } else {
                it - key
            }
        }
    }

    private fun filterSelectedProvidersByOwnership(
        selectedProviders: List<Provider>,
        selectedOwnership: Ownership?,
    ): List<Provider> =
        if (selectedOwnership == null) selectedProviders
        else selectedProviders.filter { it.ownership == selectedOwnership }

    fun removeOwnerFilter() {
        viewModelScope.launch { relayListFilterRepository.updateSelectedOwnership(Constraint.Any) }
    }

    fun removeProviderFilter() {
        viewModelScope.launch { relayListFilterRepository.updateSelectedProviders(Constraint.Any) }
    }

    fun addLocationToList(item: RelayItem.Location, customList: RelayItem.CustomList) {
        viewModelScope.launch {
            val newLocations =
                (customList.locations + item).filter { it !in item.descendants() }.map { it.id }
            val result =
                customListActionUseCase(
                        CustomListAction.UpdateLocations(customList.id, newLocations)
                    )
                    .fold(
                        { CustomListActionResultData.GenericError },
                        {
                            if (it.removedLocations.isEmpty()) {
                                CustomListActionResultData.Success.LocationAdded(
                                    customListName = it.name,
                                    locationName = item.name,
                                    undo = it.undo,
                                )
                            } else {
                                CustomListActionResultData.Success.LocationChanged(
                                    customListName = it.name,
                                    undo = it.undo,
                                )
                            }
                        },
                    )
            _uiSideEffect.send(SelectLocationSideEffect.CustomListActionToast(result))
        }
    }

    fun performAction(action: CustomListAction) {
        viewModelScope.launch { customListActionUseCase(action) }
    }

    fun removeLocationFromList(item: RelayItem.Location, customListId: CustomListId) {
        viewModelScope.launch {
            val result =
                either {
                        val customList =
                            customListsRepository.getCustomListById(customListId).bind()
                        val newLocations = (customList.locations - item.id)
                        val success =
                            customListActionUseCase(
                                    CustomListAction.UpdateLocations(customList.id, newLocations)
                                )
                                .bind()
                        if (success.addedLocations.isEmpty()) {
                            CustomListActionResultData.Success.LocationRemoved(
                                customListName = success.name,
                                locationName = item.name,
                                undo = success.undo,
                            )
                        } else {
                            CustomListActionResultData.Success.LocationChanged(
                                customListName = success.name,
                                undo = success.undo,
                            )
                        }
                    }
                    .getOrElse { CustomListActionResultData.GenericError }
            _uiSideEffect.send(SelectLocationSideEffect.CustomListActionToast(result))
        }
    }

    fun selectRelayList(relayListSelection: RelayListSelection) {
        viewModelScope.launch {
            _relayListSelection.emit(relayListSelection)
            _expandedItems.emit(initialExpand())
        }
    }

    private fun SelectedLocation.getForRelayListSelect(relayListSelection: RelayListSelection) =
        when (this) {
            is SelectedLocation.Multiple ->
                when (relayListSelection) {
                    RelayListSelection.Entry -> entryLocation
                    RelayListSelection.Exit -> exitLocation
                }.getOrNull()
            is SelectedLocation.Single -> exitLocation.getOrNull()
        }
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect

    data class CustomListActionToast(val resultData: CustomListActionResultData) :
        SelectLocationSideEffect

    data object GenericError : SelectLocationSideEffect
}

package net.mullvad.mullvadvpn.viewmodel.location

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState
import net.mullvad.mullvadvpn.compose.state.LocationBottomSheetUiState
import net.mullvad.mullvadvpn.compose.state.SetAsState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.relaylist.withDescendants
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.HopSelectionUseCase
import net.mullvad.mullvadvpn.usecase.ModifyAndEnableMultihopUseCase
import net.mullvad.mullvadvpn.usecase.ModifyMultihopError
import net.mullvad.mullvadvpn.usecase.ModifyMultihopUseCase
import net.mullvad.mullvadvpn.usecase.MultihopChange
import net.mullvad.mullvadvpn.usecase.RelayItemCanBeSelectedUseCase
import net.mullvad.mullvadvpn.usecase.SelectAndEnableMultihopUseCase
import net.mullvad.mullvadvpn.usecase.SelectRelayItemError
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.Lc

class LocationBottomSheetViewModel(
    private val locationBottomSheetState: LocationBottomSheetState,
    private val hopSelectionUseCase: HopSelectionUseCase,
    private val modifyMultihopUseCase: ModifyMultihopUseCase,
    private val modifyAndEnableMultihopUseCase: ModifyAndEnableMultihopUseCase,
    private val selectAndEnableMultihopUseCase: SelectAndEnableMultihopUseCase,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    canBeSelectedUseCase: RelayItemCanBeSelectedUseCase,
    customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    selectedLocationUseCase: SelectedLocationUseCase,
) : ViewModel() {
    val uiState: StateFlow<Lc<Unit, LocationBottomSheetUiState>> =
        combine(
                canBeSelectedUseCase(locationBottomSheetState.relayListType).take(1),
                customListsRelayItemUseCase(),
                selectedLocationUseCase().take(1),
            ) { canBeSelectedAs, customLists, selectedLocation ->
                when (locationBottomSheetState) {
                    is LocationBottomSheetState.ShowCustomListsEntryBottomSheet ->
                        Lc.Content(
                            LocationBottomSheetUiState.CustomListsEntry(
                                item = locationBottomSheetState.item,
                                setAsEntryState =
                                    canBeSelectedAs.entryIds?.validate(
                                        locationBottomSheetState.item
                                    ) ?: SetAsState.HIDDEN,
                                setAsExitState =
                                    canBeSelectedAs.exitIds?.validate(locationBottomSheetState.item)
                                        ?: SetAsState.HIDDEN,
                                // Custom list entries are never considered to be selected
                                canDisableMultihop = false,
                                customListId = locationBottomSheetState.customListId,
                                customListName =
                                    CustomListName.fromString(
                                        customLists
                                            .firstOrNull {
                                                it.id == locationBottomSheetState.customListId
                                            }
                                            ?.name ?: ""
                                    ),
                            )
                        )
                    is LocationBottomSheetState.ShowEditCustomListBottomSheet ->
                        Lc.Content(
                            LocationBottomSheetUiState.CustomList(
                                item = locationBottomSheetState.item,
                                setAsEntryState =
                                    canBeSelectedAs.entryIds?.validate(
                                        locationBottomSheetState.item
                                    ) ?: SetAsState.HIDDEN,
                                setAsExitState =
                                    canBeSelectedAs.exitIds?.validate(locationBottomSheetState.item)
                                        ?: SetAsState.HIDDEN,
                                canDisableMultihop =
                                    selectedLocation.entryLocation()?.getOrNull() ==
                                        locationBottomSheetState.item.id,
                            )
                        )
                    is LocationBottomSheetState.ShowLocationBottomSheet ->
                        Lc.Content(
                            LocationBottomSheetUiState.Location(
                                item = locationBottomSheetState.item,
                                customLists = customLists,
                                setAsEntryState =
                                    canBeSelectedAs.entryIds?.validate(
                                        locationBottomSheetState.item
                                    ) ?: SetAsState.HIDDEN,
                                setAsExitState =
                                    canBeSelectedAs.exitIds?.validate(locationBottomSheetState.item)
                                        ?: SetAsState.HIDDEN,
                                canDisableMultihop =
                                    selectedLocation.entryLocation()?.getOrNull() ==
                                        locationBottomSheetState.item.id,
                            )
                        )
                }
            }
            .stateIn(
                viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    fun setAsEntry(
        item: RelayItem,
        onError: (ModifyMultihopError, MultihopChange) -> Unit,
        onUpdateMultihop: (UndoChangeMultihopAction) -> Unit,
    ) {
        viewModelScope.launch(context = Dispatchers.IO) {
            val previousEntry =
                wireguardConstraintsRepository.wireguardConstraints.value
                    ?.entryLocation
                    ?.getOrNull()
            val change = MultihopChange.Entry(item)
            val isMultihopEnabled = isMultihopEnabled()
            if (isMultihopEnabled) {
                    modifyMultihopUseCase(change = change)
                } else {
                    modifyAndEnableMultihopUseCase(change = change, enableMultihop = true)
                }
                .fold(
                    { onError(it, change) },
                    {
                        if (!isMultihopEnabled) {
                            onUpdateMultihop(
                                if (previousEntry != null) {
                                    UndoChangeMultihopAction.DisableAndSetEntry(previousEntry)
                                } else {
                                    UndoChangeMultihopAction.Disable
                                }
                            )
                        }
                    },
                )
        }
    }

    fun setAsExit(
        item: RelayItem,
        onModifyMultihopError: (ModifyMultihopError, MultihopChange) -> Unit,
        onRelayItemError: (SelectRelayItemError) -> Unit,
        onUpdateMultihop: (UndoChangeMultihopAction) -> Unit,
    ) {
        viewModelScope.launch(context = Dispatchers.IO) {
            val previousExit = hopSelectionUseCase().first().exit()?.getOrNull()
            val isMultihopEnabled = isMultihopEnabled()
            if (isMultihopEnabled) {
                    modifyMultihopUseCase(MultihopChange.Exit(item = item))
                } else {
                    // If we are in singlehop mode we want to set a new multihop were the previous
                    // exit is set as an entry, and the new exit is set as exit. After that we turn
                    // on multihop
                    selectAndEnableMultihopUseCase(entry = previousExit, exit = item)
                }
                .fold(
                    { error ->
                        when (error) {
                            is ModifyMultihopError ->
                                onModifyMultihopError(error, MultihopChange.Exit(item))
                            is SelectRelayItemError -> onRelayItemError(error)
                            else -> error("Error not supported")
                        }
                    },
                    {
                        if (!isMultihopEnabled) {
                            onUpdateMultihop(
                                if (previousExit != null) {
                                    UndoChangeMultihopAction.DisableAndSetExit(previousExit.id)
                                } else {
                                    UndoChangeMultihopAction.Disable
                                }
                            )
                        }
                    },
                )
        }
    }

    fun disableMultihop(onUpdateMultihop: (UndoChangeMultihopAction) -> Unit) {
        viewModelScope.launch {
            wireguardConstraintsRepository
                .setMultihop(false)
                .fold(
                    { Logger.e("Set multihop error $it") },
                    { onUpdateMultihop(UndoChangeMultihopAction.Enable) },
                )
        }
    }

    private fun isMultihopEnabled() =
        wireguardConstraintsRepository.wireguardConstraints.value?.isMultihopEnabled ?: false

    private fun Set<GeoLocationId>.validate(relayItem: RelayItem): SetAsState =
        if (
            when (relayItem) {
                is RelayItem.Location -> this.contains(relayItem.id)
                is RelayItem.CustomList ->
                    relayItem.locations.withDescendants().any { this.contains(it.id) }
            }
        ) {
            SetAsState.ENABLED
        } else {
            SetAsState.DISABLED
        }
}

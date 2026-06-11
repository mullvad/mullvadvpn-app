package net.mullvad.mullvadvpn.feature.location.impl.bottomsheet

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.feature.location.api.LocationBottomSheetNavResult
import net.mullvad.mullvadvpn.feature.location.api.LocationBottomSheetState
import net.mullvad.mullvadvpn.feature.location.api.UndoChangeMultihopAction
import net.mullvad.mullvadvpn.feature.location.impl.addLocationToCustomList
import net.mullvad.mullvadvpn.feature.location.impl.removeLocationFromCustomList
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.relaylist.withDescendants
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.repository.CustomListsRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.lib.usecase.HopSelectionUseCase
import net.mullvad.mullvadvpn.lib.usecase.ModifyAndEnableMultihopUseCase
import net.mullvad.mullvadvpn.lib.usecase.ModifyMultihopError
import net.mullvad.mullvadvpn.lib.usecase.ModifyMultihopUseCase
import net.mullvad.mullvadvpn.lib.usecase.MultihopActiveUseCase
import net.mullvad.mullvadvpn.lib.usecase.RelayItemCanBeSelectedUseCase
import net.mullvad.mullvadvpn.lib.usecase.RelayMultihopChange
import net.mullvad.mullvadvpn.lib.usecase.SelectAndEnableMultihopUseCase
import net.mullvad.mullvadvpn.lib.usecase.SelectRelayItemError
import net.mullvad.mullvadvpn.lib.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.lib.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.lib.usecase.customlists.CustomListsRelayItemUseCase

@Suppress("TooManyFunctions", "LongParameterList")
class LocationBottomSheetViewModel(
    private val locationBottomSheetState: LocationBottomSheetState,
    private val customListActionUseCase: CustomListActionUseCase,
    private val customListsRepository: CustomListsRepository,
    private val hopSelectionUseCase: HopSelectionUseCase,
    private val modifyMultihopUseCase: ModifyMultihopUseCase,
    private val modifyAndEnableMultihopUseCase: ModifyAndEnableMultihopUseCase,
    private val selectAndEnableMultihopUseCase: SelectAndEnableMultihopUseCase,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val multihopActiveUseCase: MultihopActiveUseCase,
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

    private val _uiSideEffect = Channel<LocationBottomSheetNavResult>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun setAsEntry(
        item: RelayItem,
        onError: (ModifyMultihopError, RelayMultihopChange) -> Unit,
        onUpdateMultihop: (UndoChangeMultihopAction) -> Unit,
    ) {
        viewModelScope.launch(context = Dispatchers.IO) {
            val previousEntry =
                wireguardConstraintsRepository.wireguardConstraints.value
                    ?.entryLocation
                    ?.getOrNull()
            val change = RelayMultihopChange.Entry(item)
            val isMultihopActive = isMultihopActive()
            if (isMultihopActive) {
                    modifyMultihopUseCase(change = change)
                } else {
                    modifyAndEnableMultihopUseCase(
                        change = change,
                        multihopMode = MultihopMode.ALWAYS,
                    )
                }
                .fold(
                    { onError(it, change) },
                    {
                        if (!isMultihopActive) {
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
        onModifyMultihopError: (ModifyMultihopError, RelayMultihopChange) -> Unit,
        onRelayItemError: (SelectRelayItemError) -> Unit,
        onUpdateMultihop: (UndoChangeMultihopAction) -> Unit,
    ) {
        viewModelScope.launch(context = Dispatchers.IO) {
            val previousExit = hopSelectionUseCase().first().exit()?.getOrNull()
            val isMultihopActive = isMultihopActive()
            if (isMultihopActive) {
                    modifyMultihopUseCase(RelayMultihopChange.Exit(item = item))
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
                                onModifyMultihopError(error, RelayMultihopChange.Exit(item))
                            is SelectRelayItemError -> onRelayItemError(error)
                            else -> error("Error not supported")
                        }
                    },
                    {
                        if (!isMultihopActive) {
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
                .setMultihop(MultihopMode.NEVER)
                .fold(
                    { Logger.e("Set multihop error $it") },
                    { onUpdateMultihop(UndoChangeMultihopAction.Enable) },
                )
        }
    }

    fun addLocationToList(item: RelayItem.Location, customList: RelayItem.CustomList) {
        viewModelScope.launch {
            val result =
                addLocationToCustomList(
                    item = item,
                    customList = customList,
                    update = customListActionUseCase::invoke,
                )
            _uiSideEffect.send(LocationBottomSheetNavResult.CustomListActionToast(result))
        }
    }

    fun removeLocationFromList(item: RelayItem.Location, customListId: CustomListId) {
        viewModelScope.launch {
            val result =
                removeLocationFromCustomList(
                    item = item,
                    customListId = customListId,
                    getCustomListById = customListsRepository::getCustomListById,
                    update = customListActionUseCase::invoke,
                )
            _uiSideEffect.trySend(LocationBottomSheetNavResult.CustomListActionToast(result))
        }
    }

    fun onModifyMultihopError(
        modifyMultihopError: ModifyMultihopError,
        multihopChange: RelayMultihopChange,
    ) {
        viewModelScope.launch {
            _uiSideEffect.send(modifyMultihopError.toSideEffect(multihopChange))
        }
    }

    fun onSelectRelayItemError(selectRelayItemError: SelectRelayItemError) {
        viewModelScope.launch { _uiSideEffect.send(selectRelayItemError.toSideEffect()) }
    }

    fun onMultihopChanged(undoChangeMultihopAction: UndoChangeMultihopAction) {
        viewModelScope.launch {
            _uiSideEffect.send(
                LocationBottomSheetNavResult.MultihopChanged(undoChangeMultihopAction)
            )
        }
    }

    private fun ModifyMultihopError.toSideEffect(
        multihopChange: RelayMultihopChange
    ): LocationBottomSheetNavResult =
        when (this) {
            is ModifyMultihopError.EntrySameAsExit ->
                when (multihopChange) {
                    is RelayMultihopChange.Entry ->
                        LocationBottomSheetNavResult.ExitAlreadySelected(relayItem = relayItem)
                    is RelayMultihopChange.Exit ->
                        LocationBottomSheetNavResult.EntryAlreadySelected(relayItem = relayItem)
                }
            ModifyMultihopError.GenericError -> LocationBottomSheetNavResult.GenericError
            is ModifyMultihopError.RelayItemInactive ->
                LocationBottomSheetNavResult.RelayItemInactive(relayItem = relayItem)
        }

    private fun SelectRelayItemError.toSideEffect(): LocationBottomSheetNavResult =
        when (this) {
            SelectRelayItemError.GenericError -> LocationBottomSheetNavResult.GenericError
            is SelectRelayItemError.RelayInactive ->
                LocationBottomSheetNavResult.RelayItemInactive(relayItem = relayItem)
            SelectRelayItemError.EntryAndExitSame ->
                LocationBottomSheetNavResult.EntryAndExitAreSame
        }

    private suspend fun isMultihopActive(): Boolean = multihopActiveUseCase().first().isActive

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

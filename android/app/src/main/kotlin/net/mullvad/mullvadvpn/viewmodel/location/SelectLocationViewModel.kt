package net.mullvad.mullvadvpn.viewmodel.location

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.screen.location.RelayListScrollConnection
import net.mullvad.mullvadvpn.compose.screen.location.ScrollEvent
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.usecase.HopSelectionUseCase
import net.mullvad.mullvadvpn.usecase.ModifyMultihopError
import net.mullvad.mullvadvpn.usecase.ModifyMultihopUseCase
import net.mullvad.mullvadvpn.usecase.MultihopChange
import net.mullvad.mullvadvpn.usecase.SelectRelayItemError
import net.mullvad.mullvadvpn.usecase.SelectSinglehopUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.combine
import net.mullvad.mullvadvpn.util.isMultihopEnabled

@OptIn(ExperimentalCoroutinesApi::class)
@Suppress("TooManyFunctions", "LongParameterList")
class SelectLocationViewModel(
    private val relayListFilterRepository: RelayListFilterRepository,
    private val customListsRepository: CustomListsRepository,
    private val customListActionUseCase: CustomListActionUseCase,
    private val relayListRepository: RelayListRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val filterChipUseCase: FilterChipUseCase,
    private val settingsRepository: SettingsRepository,
    private val selectSingleUseCase: SelectSinglehopUseCase,
    private val modifyMultihopUseCase: ModifyMultihopUseCase,
    private val relayListScrollConnection: RelayListScrollConnection,
    hopSelectionUseCase: HopSelectionUseCase,
    connectionProxy: ConnectionProxy,
) : ViewModel() {
    private val _multihopRelayListTypeSelection: MutableStateFlow<MultihopRelayListType> =
        MutableStateFlow(MultihopRelayListType.EXIT)

    val uiState =
        combine(
                filterChips(),
                _multihopRelayListTypeSelection.filterNotNull(),
                relayListRepository.relayList,
                settingsRepository.settingsUpdates.filterNotNull(),
                connectionProxy.tunnelState
                    .map { it as? TunnelState.Error }
                    .map { it?.errorState?.cause },
                hopSelectionUseCase(),
            ) { filterChips, relayListSelection, relayList, settings, errorStateCause, selectedHop
                ->
                Lc.Content(
                    SelectLocationUiState(
                        filterChips = filterChips,
                        multihopListSelection = relayListSelection,
                        isSearchButtonEnabled =
                            searchButtonEnabled(
                                relayList = relayList,
                                relayListSelection = relayListSelection,
                                settings = settings,
                            ),
                        isFilterButtonEnabled = relayList.isNotEmpty(),
                        isRecentsEnabled = settings.recents is Recents.Enabled,
                        hopSelection = selectedHop,
                        tunnelErrorStateCause = errorStateCause,
                        entrySelectionAllowed = !settings.entryBlocked(),
                    )
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    private val _uiSideEffect = Channel<SelectLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun filterChips() =
        combine(settingsRepository.settingsUpdates, _multihopRelayListTypeSelection) {
                settings,
                multihopRelayListType ->
                if (settings?.isMultihopEnabled() == true)
                    RelayListType.Multihop(multihopRelayListType)
                else RelayListType.Single
            }
            .flatMapLatest { filterChipUseCase(it) }

    private fun searchButtonEnabled(
        relayList: List<RelayItem.Location.Country>,
        relayListSelection: MultihopRelayListType,
        settings: Settings,
    ): Boolean {
        val hasRelayListItems = relayList.isNotEmpty()
        val isEntryAndBlocked =
            isEntryAndBlocked(multihopRelayListType = relayListSelection, settings = settings)
        return hasRelayListItems && !isEntryAndBlocked
    }

    fun selectRelayList(multihopRelayListType: MultihopRelayListType) {
        viewModelScope.launch { _multihopRelayListTypeSelection.emit(multihopRelayListType) }
    }

    fun selectSingle(item: RelayItem) {
        viewModelScope.launch {
            selectSingleUseCase(item)
                .fold(
                    { _uiSideEffect.send(it.toSideEffect()) },
                    { _uiSideEffect.send(SelectLocationSideEffect.CloseScreen) },
                )
        }
    }

    fun modifyMultihop(relayItem: RelayItem, multihopRelayListType: MultihopRelayListType) {
        val change =
            when (multihopRelayListType) {
                MultihopRelayListType.ENTRY -> MultihopChange.Entry(relayItem)
                MultihopRelayListType.EXIT -> MultihopChange.Exit(relayItem)
            }

        viewModelScope.launch { modifyMultihop(change = change) }
    }

    private suspend fun modifyMultihop(change: MultihopChange) {
        modifyMultihopUseCase(change)
            .fold(
                { _uiSideEffect.send(it.toSideEffect(change)) },
                {
                    when (change) {
                        is MultihopChange.Entry ->
                            _multihopRelayListTypeSelection.emit(MultihopRelayListType.EXIT)
                        is MultihopChange.Exit ->
                            _uiSideEffect.send(SelectLocationSideEffect.CloseScreen)
                    }
                },
            )
    }

    fun addLocationToList(item: RelayItem.Location, customList: RelayItem.CustomList) {
        viewModelScope.launch {
            val result =
                addLocationToCustomList(
                    item = item,
                    customList = customList,
                    update = customListActionUseCase::invoke,
                )
            _uiSideEffect.send(SelectLocationSideEffect.CustomListActionToast(result))
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
            _uiSideEffect.trySend(SelectLocationSideEffect.CustomListActionToast(result))
        }
    }

    fun performAction(action: CustomListAction) {
        viewModelScope.launch { customListActionUseCase(action) }
    }

    fun removeOwnerFilter(relayListType: RelayListType) {
        viewModelScope.launch {
            relayListFilterRepository.updateSelectedOwnership(
                relayListType = relayListType,
                ownership = Constraint.Any,
            )
        }
    }

    fun removeProviderFilter(relayListType: RelayListType) {
        viewModelScope.launch {
            relayListFilterRepository.updateSelectedProviders(
                relayListType = relayListType,
                providers = Constraint.Any,
            )
        }
    }

    fun toggleRecentsEnabled() {
        viewModelScope.launch {
            val enabled = settingsRepository.settingsUpdates.value?.recents is Recents.Enabled
            settingsRepository.setRecentsEnabled(!enabled)
        }
    }

    fun refreshRelayList() {
        viewModelScope.launch {
            relayListRepository.refreshRelayList()
            _uiSideEffect.send(SelectLocationSideEffect.RelayListUpdating)
        }
    }

    fun toggleMultihop(enable: Boolean) {
        viewModelScope.launch {
            wireguardConstraintsRepository
                .setMultihop(enable)
                .fold(
                    { _uiSideEffect.send(SelectLocationSideEffect.GenericError) },
                    {
                        if (enable) {
                            _multihopRelayListTypeSelection.emit(MultihopRelayListType.EXIT)
                        }
                    },
                )
        }
    }

    fun scrollToItem(event: ScrollEvent) {
        relayListScrollConnection.scrollEvents.trySend(event)
    }

    fun onModifyMultihopError(
        modifyMultihopError: ModifyMultihopError,
        multihopChange: MultihopChange,
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
            _uiSideEffect.send(SelectLocationSideEffect.MultihopChanged(undoChangeMultihopAction))
        }
    }

    fun undoMultihopAction(undoChangeMultihopAction: UndoChangeMultihopAction) {
        viewModelScope.launch {
            when (undoChangeMultihopAction) {
                UndoChangeMultihopAction.Enable ->
                    wireguardConstraintsRepository.setMultihop(true).onLeft {
                        _uiSideEffect.send(SelectLocationSideEffect.GenericError)
                    }
                UndoChangeMultihopAction.Disable ->
                    wireguardConstraintsRepository.setMultihop(false).onLeft {
                        _uiSideEffect.send(SelectLocationSideEffect.GenericError)
                    }
                is UndoChangeMultihopAction.DisableAndSetEntry ->
                    wireguardConstraintsRepository
                        .setMultihopAndEntryLocation(false, undoChangeMultihopAction.relayItemId)
                        .onLeft { _uiSideEffect.send(SelectLocationSideEffect.GenericError) }
                is UndoChangeMultihopAction.DisableAndSetExit ->
                    relayListRepository
                        .updateExitRelayLocationMultihop(
                            false,
                            undoChangeMultihopAction.relayItemId,
                        )
                        .onLeft { _uiSideEffect.send(SelectLocationSideEffect.GenericError) }
            }
        }
    }

    private fun ModifyMultihopError.toSideEffect(
        multihopChange: MultihopChange
    ): SelectLocationSideEffect =
        when (this) {
            is ModifyMultihopError.EntrySameAsExit ->
                when (multihopChange) {
                    is MultihopChange.Entry ->
                        SelectLocationSideEffect.ExitAlreadySelected(relayItem = relayItem)
                    is MultihopChange.Exit ->
                        SelectLocationSideEffect.EntryAlreadySelected(relayItem = relayItem)
                }
            ModifyMultihopError.GenericError -> SelectLocationSideEffect.GenericError
            is ModifyMultihopError.RelayItemInactive ->
                SelectLocationSideEffect.RelayItemInactive(relayItem = relayItem)
        }

    private fun SelectRelayItemError.toSideEffect(): SelectLocationSideEffect =
        when (this) {
            SelectRelayItemError.GenericError -> SelectLocationSideEffect.GenericError
            is SelectRelayItemError.RelayInactive ->
                SelectLocationSideEffect.RelayItemInactive(relayItem = relayItem)
            SelectRelayItemError.EntryAndExitSame -> SelectLocationSideEffect.EntryAndExitAreSame
        }
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect

    data class CustomListActionToast(val resultData: CustomListActionResultData) :
        SelectLocationSideEffect

    data object GenericError : SelectLocationSideEffect

    data class RelayItemInactive(val relayItem: RelayItem) : SelectLocationSideEffect

    data class EntryAlreadySelected(val relayItem: RelayItem) : SelectLocationSideEffect

    data class ExitAlreadySelected(val relayItem: RelayItem) : SelectLocationSideEffect

    data object EntryAndExitAreSame : SelectLocationSideEffect

    data object RelayListUpdating : SelectLocationSideEffect

    data class MultihopChanged(val undoChangeMultihopAction: UndoChangeMultihopAction) :
        SelectLocationSideEffect
}

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
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayItem
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
import net.mullvad.mullvadvpn.usecase.SelectMultiHopUseCase
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
    private val selectMultiHopUseCase: SelectMultiHopUseCase,
    private val hopSelectionUseCase: HopSelectionUseCase,
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

    fun modifyMultihop(
        relayItem: RelayItem,
        multihopRelayListType: MultihopRelayListType,
        actionOnSuccess: Boolean = true,
    ) {
        val change =
            when (multihopRelayListType) {
                MultihopRelayListType.ENTRY -> MultihopChange.Entry(relayItem)
                MultihopRelayListType.EXIT -> MultihopChange.Exit(relayItem)
            }

        viewModelScope.launch { modifyMultihop(change = change, actionOnSuccess = actionOnSuccess) }
    }

    private suspend fun modifyMultihop(
        change: MultihopChange,
        actionOnSuccess: Boolean = true,
        onSuccess: suspend () -> Unit = {},
    ) {
        modifyMultihopUseCase(change)
            .fold(
                { _uiSideEffect.send(it.toSideEffect(change)) },
                {
                    if (actionOnSuccess) {
                        when (change) {
                            is MultihopChange.Entry ->
                                _multihopRelayListTypeSelection.emit(MultihopRelayListType.EXIT)

                            is MultihopChange.Exit -> {
                                _uiSideEffect.send(SelectLocationSideEffect.CloseScreen)
                            }
                        }
                    }
                    onSuccess()
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

    fun removeOwnerFilter() {
        viewModelScope.launch { relayListFilterRepository.updateSelectedOwnership(Constraint.Any) }
    }

    fun removeProviderFilter() {
        viewModelScope.launch { relayListFilterRepository.updateSelectedProviders(Constraint.Any) }
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

    fun setAsEntry(item: RelayItem) {
        viewModelScope.launch {
            val previousEntry = hopSelectionUseCase().first().entry()?.getOrNull()
            modifyMultihop(MultihopChange.Entry(item), actionOnSuccess = false) {
                // If we were successful we should set multihop to true if required
                if (
                    wireguardConstraintsRepository.wireguardConstraints.value?.isMultihopEnabled ==
                        false
                ) {
                    toggleMultihop(enable = true, showSnackbar = false)
                    _uiSideEffect.send(
                        SelectLocationSideEffect.MultihopChanged(
                            true,
                            revertMultihopChange =
                                previousEntry?.let { MultihopChange.Entry(previousEntry) },
                        )
                    )
                }
            }
        }
    }

    fun setAsExit(item: RelayItem) {
        viewModelScope.launch {
            if (
                wireguardConstraintsRepository.wireguardConstraints.value?.isMultihopEnabled ==
                    false
            ) {
                // If we are in singlehop mode we want to set a new multihop were the previous exit
                // is set as an entry, and the new exit is set as exit
                // After that we turn on multihop
                val previousSelection = hopSelectionUseCase().first().exit()?.getOrNull()
                if (previousSelection != null) {
                    selectMultiHopUseCase(entry = previousSelection, exit = item)
                        .fold(
                            { _uiSideEffect.send(it.toSideEffect()) },
                            {
                                toggleMultihop(enable = true, showSnackbar = false)
                                _uiSideEffect.send(
                                    SelectLocationSideEffect.MultihopChanged(
                                        true,
                                        revertMultihopChange =
                                            MultihopChange.Exit(previousSelection),
                                    )
                                )
                            },
                        )
                } else {
                    toggleMultihop(enable = true, showSnackbar = true)
                }
            } else {
                modifyMultihop(change = MultihopChange.Exit(item), actionOnSuccess = false)
            }
        }
    }

    fun toggleMultihop(
        enable: Boolean,
        showSnackbar: Boolean = false,
        onSuccess: suspend () -> Unit = {},
    ) {
        viewModelScope.launch {
            wireguardConstraintsRepository
                .setMultihop(enable)
                .fold(
                    { _uiSideEffect.send(SelectLocationSideEffect.GenericError) },
                    {
                        if (enable) {
                            _multihopRelayListTypeSelection.emit(MultihopRelayListType.EXIT)
                        }
                        if (showSnackbar) {
                            _uiSideEffect.send(SelectLocationSideEffect.MultihopChanged(enable))
                        }
                        onSuccess()
                    },
                )
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

    data class MultihopChanged(
        val enabled: Boolean,
        val revertMultihopChange: MultihopChange? = null,
    ) : SelectLocationSideEffect
}

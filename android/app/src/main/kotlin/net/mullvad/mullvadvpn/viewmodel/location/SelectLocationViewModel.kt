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
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.usecase.ModifyMultihopError
import net.mullvad.mullvadvpn.usecase.ModifyMultihopUseCase
import net.mullvad.mullvadvpn.usecase.MultihopChange
import net.mullvad.mullvadvpn.usecase.SelectHopError
import net.mullvad.mullvadvpn.usecase.SelectHopUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.combine
import net.mullvad.mullvadvpn.util.onFirst

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
    private val selectHopUseCase: SelectHopUseCase,
    private val modifyMultihopUseCase: ModifyMultihopUseCase,
    private val selectedLocationRelayItemUseCase: SelectedLocationRelayItemUseCase,
    connectionProxy: ConnectionProxy,
) : ViewModel() {
    private val _relayListType: MutableStateFlow<RelayListType?> = MutableStateFlow(null)

    val uiState =
        combine(
                filterChips(),
                wireguardConstraintsRepository.wireguardConstraints.filterNotNull().onFirst {
                    _relayListType.emit(it.initialRelayListType())
                },
                _relayListType.filterNotNull(),
                relayListRepository.relayList,
                settingsRepository.settingsUpdates.filterNotNull(),
                connectionProxy.tunnelState
                    .map { it as? TunnelState.Error }
                    .map { it?.errorState?.cause },
                selectedLocationRelayItemUseCase(),
            ) {
                filterChips,
                wireguardConstraints,
                relayListSelection,
                relayList,
                settings,
                errorStateCause,
                (entryRelayItem, exitRelayItem) ->
                Lc.Content(
                    SelectLocationUiState(
                        filterChips =
                            // Hide filter chips when entry and blocked
                            if (relayListSelection.isEntryAndBlocked(settings)) {
                                filterChips.let {
                                    val ret = it.toMutableMap()
                                    ret.put(
                                        RelayListType.Multihop(MultihopRelayListType.ENTRY),
                                        emptyList(),
                                    )
                                    ret
                                }
                            } else {
                                filterChips
                            },
                        multihopEnabled = wireguardConstraints.isMultihopEnabled,
                        relayListType = relayListSelection,
                        isSearchButtonEnabled =
                            searchButtonEnabled(
                                relayList = relayList,
                                relayListSelection = relayListSelection,
                                settings = settings,
                            ),
                        isFilterButtonEnabled = relayList.isNotEmpty(),
                        isRecentsEnabled = settings.recents is Recents.Enabled,
                        entrySelection = entryRelayItem?.name,
                        exitSelection = exitRelayItem?.name,
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
        combine(
            filterChipUseCase(RelayListType.Single),
            filterChipUseCase(RelayListType.Multihop(MultihopRelayListType.ENTRY)),
            filterChipUseCase(RelayListType.Multihop(MultihopRelayListType.EXIT)),
        ) { single, entry, exit ->
            mapOf(
                RelayListType.Single to single,
                RelayListType.Multihop(MultihopRelayListType.ENTRY) to entry,
                RelayListType.Multihop(MultihopRelayListType.EXIT) to exit,
            )
        }

    private fun searchButtonEnabled(
        relayList: List<RelayItem.Location.Country>,
        relayListSelection: RelayListType,
        settings: Settings,
    ): Boolean {
        val hasRelayListItems = relayList.isNotEmpty()
        val isEntryAndBlocked = relayListSelection.isEntryAndBlocked(settings = settings)
        return hasRelayListItems && !isEntryAndBlocked
    }

    private fun WireguardConstraints.initialRelayListType(): RelayListType =
        if (isMultihopEnabled) {
            RelayListType.Multihop(MultihopRelayListType.EXIT)
        } else {
            RelayListType.Single
        }

    fun selectRelayList(multihopRelayListType: MultihopRelayListType) {
        viewModelScope.launch { _relayListType.emit(RelayListType.Multihop(multihopRelayListType)) }
    }

    fun selectHop(hop: Hop, onSuccess: suspend () -> Unit = {}) {
        viewModelScope.launch {
            selectHopUseCase(hop)
                .fold(
                    { _uiSideEffect.send(it.toSideEffect()) },
                    {
                        onSuccess()
                        _uiSideEffect.send(SelectLocationSideEffect.CloseScreen)
                    },
                )
        }
    }

    fun modifyMultihop(relayItem: RelayItem, multihopRelayListType: MultihopRelayListType) {
        val change =
            when (multihopRelayListType) {
                MultihopRelayListType.ENTRY -> MultihopChange.Entry(relayItem)
                MultihopRelayListType.EXIT -> MultihopChange.Exit(relayItem)
            }

        viewModelScope.launch { modifyMultihop(change) }
    }

    private suspend fun modifyMultihop(change: MultihopChange, onSuccess: suspend () -> Unit = {}) {
        modifyMultihopUseCase(change)
            .fold(
                { _uiSideEffect.send(it.toSideEffect(change)) },
                {
                    when (change) {
                        is MultihopChange.Entry -> {
                            _relayListType.emit(RelayListType.Multihop(MultihopRelayListType.EXIT))
                        }
                        is MultihopChange.Exit ->
                            _uiSideEffect.send(SelectLocationSideEffect.CloseScreen)
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

    fun setAsEntry(item: RelayItem) {
        viewModelScope.launch {
            modifyMultihop(MultihopChange.Entry(item)) {
                // If we were successful we should set multihop to true if required
                if (
                    wireguardConstraintsRepository.wireguardConstraints.value?.isMultihopEnabled ==
                        false
                ) {
                    setMultihop(enable = true, showSnackbar = true)
                    _uiSideEffect.send(SelectLocationSideEffect.MultihopChanged(true))
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
                val previousSelection = selectedLocationRelayItemUseCase().first().second
                if (previousSelection != null) {
                    selectHop(Hop.Multi(entry = previousSelection, exit = item)) {
                        setMultihop(enable = true, showSnackbar = true)
                    }
                } else {
                    setMultihop(enable = true, showSnackbar = true)
                }
            } else {
                modifyMultihop(MultihopChange.Exit(item))
            }
        }
    }

    fun setMultihop(enable: Boolean, showSnackbar: Boolean = false) {
        viewModelScope.launch {
            wireguardConstraintsRepository
                .setMultihop(enable)
                .fold(
                    { _uiSideEffect.send(SelectLocationSideEffect.GenericError) },
                    {
                        _relayListType.emit(
                            if (enable) {
                                RelayListType.Multihop(MultihopRelayListType.EXIT)
                            } else {
                                RelayListType.Single
                            }
                        )
                        if (showSnackbar) {
                            _uiSideEffect.send(SelectLocationSideEffect.MultihopChanged(enable))
                        }
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
                SelectLocationSideEffect.RelayItemInactive(hop = Hop.Single(this.relayItem))
        }

    private fun SelectHopError.toSideEffect(): SelectLocationSideEffect =
        when (this) {
            SelectHopError.GenericError -> SelectLocationSideEffect.GenericError
            is SelectHopError.HopInactive ->
                SelectLocationSideEffect.RelayItemInactive(hop = this.hop)
            SelectHopError.EntryAndExitSame -> SelectLocationSideEffect.EntryAndExitAreSame
        }
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect

    data class CustomListActionToast(val resultData: CustomListActionResultData) :
        SelectLocationSideEffect

    data object GenericError : SelectLocationSideEffect

    data class RelayItemInactive(val hop: Hop) : SelectLocationSideEffect

    data class EntryAlreadySelected(val relayItem: RelayItem) : SelectLocationSideEffect

    data class ExitAlreadySelected(val relayItem: RelayItem) : SelectLocationSideEffect

    data object EntryAndExitAreSame : SelectLocationSideEffect

    data object RelayListUpdating : SelectLocationSideEffect

    data class MultihopChanged(val enabled: Boolean) : SelectLocationSideEffect
}

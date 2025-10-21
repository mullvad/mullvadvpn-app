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
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
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
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.onFirst

@OptIn(ExperimentalCoroutinesApi::class)
@Suppress("TooManyFunctions")
class SelectLocationViewModel(
    private val relayListFilterRepository: RelayListFilterRepository,
    private val customListsRepository: CustomListsRepository,
    private val customListActionUseCase: CustomListActionUseCase,
    private val relayListRepository: RelayListRepository,
    wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val filterChipUseCase: FilterChipUseCase,
    private val settingsRepository: SettingsRepository,
    private val selectHopUseCase: SelectHopUseCase,
    private val modifyMultihopUseCase: ModifyMultihopUseCase,
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
            ) { filterChips, wireguardConstraints, relayListSelection, relayList, settings ->
                Lc.Content(
                    SelectLocationUiState(
                        filterChips =
                            // Hide filter chips when entry and blocked
                            if (relayListSelection.isEntryAndBlocked(settings)) {
                                emptyList()
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
        _relayListType.filterNotNull().flatMapLatest { filterChipUseCase(it) }

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

    fun selectHop(hop: Hop) {
        viewModelScope.launch {
            selectHopUseCase(hop)
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

        viewModelScope.launch {
            modifyMultihopUseCase(change)
                .fold(
                    { _uiSideEffect.send(it.toSideEffect(multihopRelayListType)) },
                    {
                        when (multihopRelayListType) {
                            MultihopRelayListType.ENTRY -> {
                                _relayListType.emit(
                                    RelayListType.Multihop(MultihopRelayListType.EXIT)
                                )
                                _uiSideEffect.send(
                                    SelectLocationSideEffect.FocusExitList(relayItem)
                                )
                            }
                            MultihopRelayListType.EXIT ->
                                _uiSideEffect.send(SelectLocationSideEffect.CloseScreen)
                        }
                    },
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
        viewModelScope.launch {
            _relayListType.value?.let {
                relayListFilterRepository.updateSelectedOwnership(Constraint.Any, it)
            }
        }
    }

    fun removeProviderFilter() {
        viewModelScope.launch {
            _relayListType.value?.let {
                relayListFilterRepository.updateSelectedProviders(Constraint.Any, it)
            }
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

    private fun ModifyMultihopError.toSideEffect(
        multihopRelayListType: MultihopRelayListType
    ): SelectLocationSideEffect =
        when (this) {
            is ModifyMultihopError.EntrySameAsExit ->
                when (multihopRelayListType) {
                    MultihopRelayListType.ENTRY ->
                        SelectLocationSideEffect.ExitAlreadySelected(relayItem = relayItem)
                    MultihopRelayListType.EXIT ->
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

    data class FocusExitList(val relayItem: RelayItem) : SelectLocationSideEffect
}

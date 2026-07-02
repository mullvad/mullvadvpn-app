package net.mullvad.mullvadvpn.feature.location.impl

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
import net.mullvad.mullvadvpn.feature.location.api.UndoChangeMultihopAction
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.combine
import net.mullvad.mullvadvpn.lib.common.util.entryBlocked
import net.mullvad.mullvadvpn.lib.common.util.isEntryAndBlocked
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.FilterTarget
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.communication.CustomListAction
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.lib.usecase.AutomaticEntryMultihopChange
import net.mullvad.mullvadvpn.lib.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.lib.usecase.HopSelectionUseCase
import net.mullvad.mullvadvpn.lib.usecase.LastKnownLocationUseCase
import net.mullvad.mullvadvpn.lib.usecase.ModifyMultihopError
import net.mullvad.mullvadvpn.lib.usecase.ModifyMultihopUseCase
import net.mullvad.mullvadvpn.lib.usecase.MultihopChange
import net.mullvad.mullvadvpn.lib.usecase.MultihopInEffectUseCase
import net.mullvad.mullvadvpn.lib.usecase.RelayMultihopChange
import net.mullvad.mullvadvpn.lib.usecase.SelectRelayItemError
import net.mullvad.mullvadvpn.lib.usecase.SelectSinglehopUseCase
import net.mullvad.mullvadvpn.lib.usecase.customlists.CustomListActionUseCase

@OptIn(ExperimentalCoroutinesApi::class)
@Suppress("TooManyFunctions", "LongParameterList")
class SelectLocationViewModel(
    private val relayListFilterRepository: RelayListFilterRepository,
    private val customListActionUseCase: CustomListActionUseCase,
    private val relayListRepository: RelayListRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val filterChipUseCase: FilterChipUseCase,
    private val settingsRepository: SettingsRepository,
    private val selectSingleUseCase: SelectSinglehopUseCase,
    private val modifyMultihopUseCase: ModifyMultihopUseCase,
    private val relayListScrollConnection: RelayListScrollConnection,
    private val multihopInEffectUseCase: MultihopInEffectUseCase,
    hopSelectionUseCase: HopSelectionUseCase,
    lastKnownLocationUseCase: LastKnownLocationUseCase,
    connectionProxy: ConnectionProxy,
) : ViewModel() {
    private val _multihopRelayListTypeSelection: MutableStateFlow<MultihopRelayListType> =
        MutableStateFlow(MultihopRelayListType.EXIT)

    val uiState =
        combine(
                filterChips(),
                _multihopRelayListTypeSelection,
                relayListRepository.relayList,
                settingsRepository.settingsUpdates.filterNotNull(),
                connectionProxy.tunnelState,
                hopSelectionUseCase(),
                lastKnownLocationUseCase.lastKnownDisconnectedLocation,
                relayListFilterRepository.hasAnyFilterFlow(),
            ) {
                filterChips,
                relayListSelection,
                relayList,
                settings,
                tunnelState,
                selectedHop,
                lastKnownLocation,
                filterState ->
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
                        hasAnyEntryFilter = filterState.hasAnyEntryFilter,
                        hasAnyExitFilter = filterState.hasAnyExitFilter,
                        tunnelErrorStateCause = tunnelState.errorCause,
                        isEntryFilteringEnabled = !settings.entryBlocked(),
                        lastKnownLocation = lastKnownLocation?.country,
                        entryCountry = tunnelState.entryCountry,
                    )
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    private val TunnelState.errorCause: ErrorStateCause?
        get() = (this as? TunnelState.Error)?.errorState?.cause

    private val TunnelState.entryCountry: String?
        get() = (this as? TunnelState.Connected)?.location?.entryCountry

    private val _uiSideEffect = Channel<SelectLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun filterChips() =
        combine(_multihopRelayListTypeSelection, multihopInEffectUseCase()) {
                multihopRelayListType,
                multihopActive ->
                if (multihopActive.isInEffect) RelayListType.Multihop(multihopRelayListType)
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
                MultihopRelayListType.ENTRY -> RelayMultihopChange.Entry(relayItem)
                MultihopRelayListType.EXIT -> RelayMultihopChange.Exit(relayItem)
            }

        viewModelScope.launch { modifyMultihop(change = change) }
    }

    private suspend fun modifyMultihop(change: MultihopChange) {
        modifyMultihopUseCase(change)
            .fold(
                {
                    when (change) {
                        is RelayMultihopChange -> _uiSideEffect.send(it.toSideEffect(change))
                        AutomaticEntryMultihopChange ->
                            _uiSideEffect.send(SelectLocationSideEffect.GenericError)
                    }
                },
                {
                    when (change) {
                        is RelayMultihopChange.Entry,
                        AutomaticEntryMultihopChange ->
                            _multihopRelayListTypeSelection.emit(MultihopRelayListType.EXIT)

                        is RelayMultihopChange.Exit ->
                            _uiSideEffect.send(SelectLocationSideEffect.CloseScreen)
                    }
                },
            )
    }

    fun selectAutomaticMultihopEntry() {
        viewModelScope.launch { modifyMultihop(AutomaticEntryMultihopChange) }
    }

    private fun ModifyMultihopError.toSideEffect(
        multihopChange: RelayMultihopChange
    ): SelectLocationSideEffect =
        when (this) {
            is ModifyMultihopError.EntrySameAsExit ->
                when (multihopChange) {
                    is RelayMultihopChange.Entry ->
                        SelectLocationSideEffect.ExitAlreadySelected(relayItem = relayItem)

                    is RelayMultihopChange.Exit ->
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

    fun performAction(action: CustomListAction) {
        viewModelScope.launch { customListActionUseCase(action) }
    }

    fun removeOwnerFilter(filterTarget: FilterTarget) {
        viewModelScope.launch {
            relayListFilterRepository.updateSelectedOwnership(Constraint.Any, filterTarget)
        }
    }

    fun removeProviderFilter(filterTarget: FilterTarget) {
        viewModelScope.launch {
            relayListFilterRepository.updateSelectedProviders(Constraint.Any, filterTarget)
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
                .setMultihop(if (enable) MultihopMode.ALWAYS else MultihopMode.NEVER)
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

    fun undoMultihopAction(undoChangeMultihopAction: UndoChangeMultihopAction) {
        viewModelScope.launch {
            when (undoChangeMultihopAction) {
                UndoChangeMultihopAction.Enable ->
                    wireguardConstraintsRepository.setMultihop(MultihopMode.ALWAYS).onLeft {
                        _uiSideEffect.send(SelectLocationSideEffect.GenericError)
                    }

                UndoChangeMultihopAction.Disable ->
                    wireguardConstraintsRepository.setMultihop(MultihopMode.NEVER).onLeft {
                        _uiSideEffect.send(SelectLocationSideEffect.GenericError)
                    }

                is UndoChangeMultihopAction.DisableAndSetEntry ->
                    wireguardConstraintsRepository
                        .setMultihopAndEntryLocation(
                            MultihopMode.NEVER,
                            undoChangeMultihopAction.relayItemId,
                        )
                        .onLeft { _uiSideEffect.send(SelectLocationSideEffect.GenericError) }

                is UndoChangeMultihopAction.DisableAndSetExit ->
                    relayListRepository
                        .updateExitRelayLocationMultihop(
                            MultihopMode.NEVER,
                            undoChangeMultihopAction.relayItemId,
                        )
                        .onLeft { _uiSideEffect.send(SelectLocationSideEffect.GenericError) }
            }
        }
    }

    fun scrollToItem(event: ScrollEvent) {
        relayListScrollConnection.scrollEvents.trySend(event)
    }
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect

    data object GenericError : SelectLocationSideEffect

    data class RelayItemInactive(val relayItem: RelayItem) : SelectLocationSideEffect

    data class EntryAlreadySelected(val relayItem: RelayItem) : SelectLocationSideEffect

    data class ExitAlreadySelected(val relayItem: RelayItem) : SelectLocationSideEffect

    data object EntryAndExitAreSame : SelectLocationSideEffect

    data object RelayListUpdating : SelectLocationSideEffect
}

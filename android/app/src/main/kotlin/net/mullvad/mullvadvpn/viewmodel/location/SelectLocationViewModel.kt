package net.mullvad.mullvadvpn.viewmodel.location

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.Recents
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.usecase.SelectHopError
import net.mullvad.mullvadvpn.usecase.SelectHopUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.util.Lc

@OptIn(ExperimentalCoroutinesApi::class)
@Suppress("TooManyFunctions")
class SelectLocationViewModel(
    private val relayListFilterRepository: RelayListFilterRepository,
    private val customListsRepository: CustomListsRepository,
    private val customListActionUseCase: CustomListActionUseCase,
    relayListRepository: RelayListRepository,
    wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val filterChipUseCase: FilterChipUseCase,
    private val settingsRepository: SettingsRepository,
    private val selectHopUseCase: SelectHopUseCase,
) : ViewModel() {
    private val _relayListType: MutableStateFlow<RelayListType> =
        MutableStateFlow(RelayListType.EXIT)

    val uiState =
        combine(
                filterChips(),
                wireguardConstraintsRepository.wireguardConstraints,
                _relayListType,
                relayListRepository.relayList,
                settingsRepository.settingsUpdates,
            ) { filterChips, wireguardConstraints, relayListSelection, relayList, settings ->
                Lc.Content(
                    SelectLocationUiState(
                        filterChips = filterChips,
                        multihopEnabled = wireguardConstraints?.isMultihopEnabled == true,
                        relayListType = relayListSelection,
                        isSearchButtonEnabled =
                            relayList.isNotEmpty() &&
                                (relayListSelection == RelayListType.EXIT ||
                                    settings?.entryBlocked() != true),
                        isFilterButtonEnabled = relayList.isNotEmpty(),
                        isRecentsEnabled = settings?.recents is Recents.Enabled,
                    )
                )
            }
            .stateIn(viewModelScope, SharingStarted.Lazily, Lc.Loading(Unit))

    private val _uiSideEffect = Channel<SelectLocationSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun filterChips() = _relayListType.flatMapLatest { filterChipUseCase(it) }

    fun selectRelayList(relayListType: RelayListType) {
        viewModelScope.launch { _relayListType.emit(relayListType) }
    }

    fun selectHop(hop: Hop, relayListType: RelayListType) {
        viewModelScope.launch {
            selectHopUseCase(hop, relayListType)
                .fold(
                    {
                        when (it) {
                            SelectHopError.GenericError ->
                                _uiSideEffect.trySend(SelectLocationSideEffect.GenericError)
                            is SelectHopError.HopNotActive ->
                                _uiSideEffect.trySend(
                                    SelectLocationSideEffect.RelayItemInactive(it.hop)
                                )
                            SelectHopError.EntryBlocked,
                            SelectHopError.ExitBlocked ->
                                _uiSideEffect.send(
                                    SelectLocationSideEffect.RelayItemAlreadySelected(
                                        hop = hop,
                                        relayListType = relayListType,
                                    )
                                )
                        }
                    },
                    {
                        when (relayListType) {
                            RelayListType.ENTRY ->
                                if (hop is Hop.Multi)
                                    _uiSideEffect.send(SelectLocationSideEffect.CloseScreen)
                                else _relayListType.emit(RelayListType.EXIT)
                            RelayListType.EXIT ->
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
}

sealed interface SelectLocationSideEffect {
    data object CloseScreen : SelectLocationSideEffect

    data class CustomListActionToast(val resultData: CustomListActionResultData) :
        SelectLocationSideEffect

    data object GenericError : SelectLocationSideEffect

    data class RelayItemInactive(val hop: Hop) : SelectLocationSideEffect

    data class RelayItemAlreadySelected(val hop: Hop, val relayListType: RelayListType) :
        SelectLocationSideEffect
}

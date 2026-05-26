package net.mullvad.mullvadvpn.feature.multihopmigration.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.feature.multihopmigration.api.MultihopMigrationNavKey
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.MultihopMigrationState
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.PreviousDaitaState
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.SplitFilterMigration
import net.mullvad.mullvadvpn.lib.repository.MultihopMigrationRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository

class MultihopMigrationViewModel(
    navArgs: MultihopMigrationNavKey,
    private val multihopMigrationRepository: MultihopMigrationRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) : ViewModel() {
    private val _uiSideEffect = Channel<MultihopMigrationScreenSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val pages = generatePages(navArgs.errorFallback, navArgs.migration)
    private val currentPage = MutableStateFlow(0)

    val uiState: StateFlow<MultihopMigrationUiState> =
        combine(flowOf(pages), currentPage, ::createState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                MultihopMigrationUiState(emptyList(), 0),
            )

    private fun createState(
        pages: List<MultihopMigrationPage>,
        page: Int,
    ): MultihopMigrationUiState =
        MultihopMigrationUiState(multihopMigrationPages = pages, currentPageIndex = page)

    private fun generatePages(
        errorFallback: Boolean,
        splitFilterMigration: SplitFilterMigration,
    ): List<MultihopMigrationPage> = buildList {
        // Order is important!
        add(MultihopMigrationPage.NewMultihopMode(splitFilterMigration.multihopMigrationState))
        if (splitFilterMigration.daitaMigration == PreviousDaitaState.DIRECT_ONLY) {
            add(MultihopMigrationPage.DirectOnlyRemoved)
        }
        if (!splitFilterMigration.filtersSet) {
            add(MultihopMigrationPage.SeparateFilters)
        }
        // If migrating to multihop always we want to suggest setting the multihop entry to
        // automatic
        if (
            splitFilterMigration.multihopMigrationState == MultihopMigrationState.ON_TO_ALWAYS ||
                splitFilterMigration.multihopMigrationState == MultihopMigrationState.OFF_TO_ALWAYS
        ) {
            add(MultihopMigrationPage.SuggestedMultihopEntry)
        }
        // There are two scenarios where we want to show the page to suggest a change to multihop
        // mode when needed:
        // - If we are on the generic error fallback flow we want to suggest setting the multihop
        // mode to when needed to unblock the user.
        // - If the user is migrating to multihop never and have filters set we want to suggest when
        // needed multihop setting to prevent the user from being blocked in the future.
        when {
            errorFallback -> add(MultihopMigrationPage.SuggestedAction)
            splitFilterMigration.multihopMigrationState == MultihopMigrationState.OFF_TO_NEVER &&
                splitFilterMigration.filtersSet -> add(MultihopMigrationPage.SuggestedAction)
        }
    }

    fun nextPage() {
        if (currentPage.value < pages.size - 1) {
            currentPage.value += 1
        }
    }

    fun previousPage() {
        if (currentPage.value > 0) {
            currentPage.value -= 1
        }
    }

    fun setEntryLocation(entry: Constraint<RelayItemId>) = viewModelScope.launch {
        wireguardConstraintsRepository.setEntryLocation(entry).onLeft {
            _uiSideEffect.send(MultihopMigrationScreenSideEffect.GenericError)
        }
    }

    fun setMultihopMode(multihopMode: MultihopMode) = viewModelScope.launch {
        wireguardConstraintsRepository.setMultihopMode(multihopMode)
    }

    fun finishMigration() = viewModelScope.launch {
        multihopMigrationRepository
            .clearMultihopMigrationState()
            .fold(
                { _uiSideEffect.send(MultihopMigrationScreenSideEffect.GenericError) },
                { _uiSideEffect.send(MultihopMigrationScreenSideEffect.CloseScreen) },
            )
    }
}

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
import net.mullvad.mullvadvpn.lib.model.MultihopMigrationData
import net.mullvad.mullvadvpn.lib.model.MultihopMigrationState
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.PreviousDaitaState
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository

class MultihopMigrationViewModel(
    navArgs: MultihopMigrationNavKey,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val userPreferencesRepository: UserPreferencesRepository,
) : ViewModel() {
    private val _uiSideEffect = Channel<MultihopMigrationScreenSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val pages = generatePages(navArgs.multihopMigrationData)
    private val currentPage = MutableStateFlow(0)

    val uiState: StateFlow<MultihopMigrationUiState> =
        combine(flowOf(pages), currentPage, ::createState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                MultihopMigrationUiState(emptyList(), 0),
            )

    init {
        viewModelScope.launch { userPreferencesRepository.setHasSeenMultihopMigrationGuide() }
    }

    private fun createState(
        pages: List<MultihopMigrationPage>,
        page: Int,
    ): MultihopMigrationUiState =
        MultihopMigrationUiState(multihopMigrationPages = pages, currentPageIndex = page)

    private fun generatePages(
        multihopMigrationData: MultihopMigrationData
    ): List<MultihopMigrationPage> = buildList {
        // Order is important!
        add(
            MultihopMigrationPage.NewMultihopMode(
                multihopMigrationData.splitFilterMigration.multihopMigrationState
            )
        )
        if (
            multihopMigrationData.splitFilterMigration.daitaMigration ==
                PreviousDaitaState.DIRECT_ONLY
        ) {
            add(MultihopMigrationPage.DirectOnlyRemoved)
        }
        if (multihopMigrationData.splitFilterMigration.filtersSet) {
            add(MultihopMigrationPage.SeparateFilters)
        }
        // If the user had multihop turned on, DAITA enabled and filters set
        // --or--
        // If the user was using magic multihop with daita and filters were set
        // we want to suggest setting automatic location as entry
        when {
            multihopMigrationData.splitFilterMigration.multihopMigrationState ==
                MultihopMigrationState.ON_TO_ALWAYS &&
                multihopMigrationData.splitFilterMigration.daitaMigration !=
                    PreviousDaitaState.OFF &&
                multihopMigrationData.splitFilterMigration.filtersSet ->
                add(MultihopMigrationPage.SuggestedMultihopEntry)
            multihopMigrationData.splitFilterMigration.multihopMigrationState ==
                MultihopMigrationState.OFF_TO_ALWAYS &&
                multihopMigrationData.splitFilterMigration.filtersSet ->
                add(MultihopMigrationPage.SuggestedMultihopEntry)
        }

        // There are three scenarios where we want to show the page to suggest a change to multihop
        // mode when needed:
        // - If we are on the generic error fallback flow we want to suggest setting the multihop
        // mode to when needed to unblock the user.
        // - If the user had neither multihop nor daita enabled and have filters set we want to
        // suggest when needed multihop setting to prevent the user from being blocked in the
        // future.
        // - If the user was using daita without direct only but had selected a serer with daita
        // support and had filters enabled we want to suggest when needed multihop to prevent being
        // blocked in the future.
        when {
            multihopMigrationData.userBlocked -> add(MultihopMigrationPage.SuggestedAction)
            multihopMigrationData.splitFilterMigration.multihopMigrationState ==
                MultihopMigrationState.OFF_TO_NEVER &&
                multihopMigrationData.splitFilterMigration.daitaMigration !=
                    PreviousDaitaState.DIRECT_ONLY &&
                multihopMigrationData.splitFilterMigration.filtersSet ->
                add(MultihopMigrationPage.SuggestedAction)
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
}

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
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.Scenario
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

    @Suppress("CyclomaticComplexMethod")
    private fun generatePages(
        multihopMigrationData: MultihopMigrationData
    ): List<MultihopMigrationPage> = buildList {
        // Scenarios to page conversion is based on the scenario flow chart.
        when (multihopMigrationData.splitFilterMigration.scenario) {
            Scenario.ONE_A -> error("Scenario A should not show the guide")
            Scenario.ONE_B -> {
                add(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_NEVER))
                add(MultihopMigrationPage.SeparateFilters)
                add(MultihopMigrationPage.SuggestedAction)
            }
            Scenario.TWO -> {
                add(
                    MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_WHEN_NEEDED)
                )
            }
            Scenario.THREE_A -> {
                add(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_NEVER))
                add(MultihopMigrationPage.SeparateFilters)
                add(MultihopMigrationPage.SuggestedAction)
            }
            Scenario.THREE_B -> {
                add(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_ALWAYS))
                add(MultihopMigrationPage.SeparateFilters)
                add(MultihopMigrationPage.SuggestedMultihopEntry)
            }
            Scenario.FOUR_A -> {
                add(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_NEVER))
                add(MultihopMigrationPage.DirectOnlyRemoved)
            }
            Scenario.FOUR_B -> {
                add(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.OFF_TO_NEVER))
                add(MultihopMigrationPage.DirectOnlyRemoved)
                add(MultihopMigrationPage.SeparateFilters)
            }
            Scenario.FIVE_A -> {
                add(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS))
            }
            Scenario.FIVE_B -> {
                add(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS))
                add(MultihopMigrationPage.SeparateFilters)
            }
            Scenario.SIX_A -> {
                add(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS))
                add(MultihopMigrationPage.EntrySetToAutomatic)
            }
            Scenario.SIX_B -> {
                add(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS))
                add(MultihopMigrationPage.SeparateFilters)
                add(MultihopMigrationPage.SuggestedMultihopEntry)
            }
            Scenario.SEVEN_A -> {
                add(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS))
                add(MultihopMigrationPage.DirectOnlyRemoved)
            }
            Scenario.SEVEN_B -> {
                add(MultihopMigrationPage.NewMultihopMode(MultihopMigrationState.ON_TO_ALWAYS))
                add(MultihopMigrationPage.DirectOnlyRemoved)
                add(MultihopMigrationPage.SeparateFilters)
                add(MultihopMigrationPage.SuggestedMultihopEntry)
            }
        }

        // If the user is blocked we always want to show the suggested action page, so we will add
        // that page if needed.
        if (multihopMigrationData.userBlocked && !contains(MultihopMigrationPage.SuggestedAction)) {
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

package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.usecase.SelectedLocationTitleUseCase
import net.mullvad.mullvadvpn.lib.userpreferences.OnboardingCheckpoint
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository

data class ReadyToConnectUiState(val selectedLocationTitle: String? = null)

class ReadyToConnectOnboardingViewModel(
    private val userPreferencesRepository: UserPreferencesRepository,
    selectedLocationTitleUseCase: SelectedLocationTitleUseCase,
) : ViewModel() {

    val uiState: StateFlow<ReadyToConnectUiState> =
        selectedLocationTitleUseCase()
            .map { ReadyToConnectUiState(selectedLocationTitle = it) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                ReadyToConnectUiState(),
            )

    fun complete() = viewModelScope.launch {
        userPreferencesRepository.setOnboardingCheckpoint(OnboardingCheckpoint.PART_1_COMPLETED)
    }
}

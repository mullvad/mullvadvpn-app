package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.usecase.SelectedLocationTitleUseCase
import net.mullvad.mullvadvpn.lib.userpreferences.UserPreferencesRepository

data class ReadyToConnectUiState(val selectedLocationTitle: String? = null)

class ReadyToConnectViewModel(
    selectedLocationTitleUseCase: SelectedLocationTitleUseCase,
    userPreferencesRepository: UserPreferencesRepository,
) : OnboardingViewModel(userPreferencesRepository) {

    val uiState: StateFlow<ReadyToConnectUiState> =
        selectedLocationTitleUseCase()
            .map { ReadyToConnectUiState(selectedLocationTitle = it) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                ReadyToConnectUiState(),
            )
}

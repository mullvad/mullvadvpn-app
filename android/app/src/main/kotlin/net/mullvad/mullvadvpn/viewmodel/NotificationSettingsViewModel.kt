package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.NotificationSettingsUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc

class NotificationSettingsViewModel(
    private val userPreferencesRepository: UserPreferencesRepository
) : ViewModel() {

    val uiState =
        userPreferencesRepository
            .preferencesFlow()
            .map { settings ->
                NotificationSettingsUiState(
                        locationInNotificationEnabled = settings.showLocationInSystemNotification
                    )
                    .toLc<Unit, NotificationSettingsUiState>()
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                initialValue = Lc.Loading(Unit),
            )

    fun onToggleLocationInNotifications(enabled: Boolean) {
        viewModelScope.launch {
            userPreferencesRepository.setLocationInNotificationEnabled(enabled)
        }
    }
}

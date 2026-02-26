package net.mullvad.mullvadvpn.feature.notification.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository

sealed interface NotificationSettingsSideEffect {
    data object OpenSystemNotificationsSettings : NotificationSettingsSideEffect
}

class NotificationSettingsViewModel(
    private val userPreferencesRepository: UserPreferencesRepository
) : ViewModel() {

    private val _uiSideEffect = Channel<NotificationSettingsSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

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
                started = SharingStarted.Companion.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                initialValue = Lc.Loading(Unit),
            )

    fun onToggleLocationInNotifications(enabled: Boolean) {
        viewModelScope.launch {
            userPreferencesRepository.setLocationInNotificationEnabled(enabled)
        }
    }

    fun openSystemNotificationsSettings() =
        viewModelScope.launch {
            _uiSideEffect.send(NotificationSettingsSideEffect.OpenSystemNotificationsSettings)
        }
}

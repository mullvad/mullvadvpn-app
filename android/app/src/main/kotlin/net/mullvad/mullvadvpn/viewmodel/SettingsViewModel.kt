package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache

class SettingsViewModel(
    deviceRepository: DeviceRepository,
    appVersionInfoCache: AppVersionInfoCache,
    isPlayBuild: Boolean
) : ViewModel() {

    private val vmState: StateFlow<SettingsUiState> =
        combine(deviceRepository.deviceState, appVersionInfoCache.versionInfo()) {
                deviceState,
                versionInfo ->
                SettingsUiState(
                    isLoggedIn = deviceState is DeviceState.LoggedIn,
                    appVersion = versionInfo.currentVersion ?: "",
                    isUpdateAvailable = versionInfo.let { it.isSupported.not() || it.isOutdated },
                    isPlayBuild = isPlayBuild
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SettingsUiState(
                    appVersion = "",
                    isLoggedIn = false,
                    isUpdateAvailable = false,
                    isPlayBuild
                )
            )

    val uiState =
        vmState.stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(),
            SettingsUiState(
                appVersion = "",
                isLoggedIn = false,
                isUpdateAvailable = false,
                isPlayBuild
            )
        )
}

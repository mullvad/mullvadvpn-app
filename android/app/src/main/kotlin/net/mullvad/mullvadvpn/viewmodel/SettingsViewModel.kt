package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoRepository

class SettingsViewModel(
    deviceRepository: DeviceRepository,
    appVersionInfoRepository: AppVersionInfoRepository,
    isPlayBuild: Boolean
) : ViewModel() {

    private val vmState: StateFlow<SettingsUiState> =
        combine(deviceRepository.deviceState, appVersionInfoRepository.versionInfo()) {
                deviceState,
                versionInfo ->
                SettingsUiState(
                    isLoggedIn = deviceState is DeviceState.LoggedIn,
                    appVersion = versionInfo.currentVersion,
                    isSupportedVersion = versionInfo.isSupported,
                    isPlayBuild = isPlayBuild
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SettingsUiState(
                    appVersion = "",
                    isLoggedIn = false,
                    isSupportedVersion = true,
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
                isSupportedVersion = true,
                isPlayBuild
            )
        )
}

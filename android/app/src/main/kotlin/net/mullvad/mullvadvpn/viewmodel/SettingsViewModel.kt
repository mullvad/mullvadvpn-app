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
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoRepository

class SettingsViewModel(
    deviceRepository: DeviceRepository,
    appVersionInfoRepository: AppVersionInfoRepository,
    wireguardConstraintsRepository: WireguardConstraintsRepository,
    isPlayBuild: Boolean,
) : ViewModel() {

    val uiState: StateFlow<SettingsUiState> =
        combine(
                deviceRepository.deviceState,
                appVersionInfoRepository.versionInfo,
                wireguardConstraintsRepository.wireguardConstraints,
            ) { deviceState, versionInfo, wireguardConstraints ->
                SettingsUiState(
                    isLoggedIn = deviceState is DeviceState.LoggedIn,
                    appVersion = versionInfo.currentVersion,
                    isSupportedVersion = versionInfo.isSupported,
                    isPlayBuild = isPlayBuild,
                    useMultihop = wireguardConstraints?.useMultihop ?: false,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SettingsUiState(
                    appVersion = "",
                    isLoggedIn = false,
                    isSupportedVersion = true,
                    isPlayBuild = isPlayBuild,
                    useMultihop = false,
                ),
            )
}

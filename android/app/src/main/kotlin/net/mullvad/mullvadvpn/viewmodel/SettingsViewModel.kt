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
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoRepository

class SettingsViewModel(
    deviceRepository: DeviceRepository,
    appVersionInfoRepository: AppVersionInfoRepository,
    wireguardConstraintsRepository: WireguardConstraintsRepository,
    settingsRepository: SettingsRepository,
    isPlayBuild: Boolean,
) : ViewModel() {

    val uiState: StateFlow<SettingsUiState> =
        combine(
                deviceRepository.deviceState,
                appVersionInfoRepository.versionInfo,
                wireguardConstraintsRepository.wireguardConstraints,
                settingsRepository.settingsUpdates,
            ) { deviceState, versionInfo, wireguardConstraints, settings ->
                SettingsUiState(
                    isLoggedIn = deviceState is DeviceState.LoggedIn,
                    appVersion = versionInfo.currentVersion,
                    isSupportedVersion = versionInfo.isSupported,
                    multihopEnabled = wireguardConstraints?.isMultihopEnabled == true,
                    isDaitaEnabled =
                        settings?.tunnelOptions?.wireguard?.daitaSettings?.enabled == true,
                    isPlayBuild = isPlayBuild,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SettingsUiState(
                    appVersion = "",
                    isLoggedIn = false,
                    isSupportedVersion = true,
                    isDaitaEnabled = false,
                    isPlayBuild = isPlayBuild,
                    multihopEnabled = false,
                ),
            )
}

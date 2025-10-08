package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoRepository
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc

class SettingsViewModel(
    deviceRepository: DeviceRepository,
    appVersionInfoRepository: AppVersionInfoRepository,
    wireguardConstraintsRepository: WireguardConstraintsRepository,
    settingsRepository: SettingsRepository,
    isPlayBuild: Boolean,
) : ViewModel() {

    val uiState: StateFlow<Lc<Unit, SettingsUiState>> =
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
                        isDaitaEnabled = settings?.tunnelOptions?.daitaSettings?.enabled == true,
                        isPlayBuild = isPlayBuild,
                    )
                    .toLc<Unit, SettingsUiState>()
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )
}

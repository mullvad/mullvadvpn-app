package net.mullvad.mullvadvpn.feature.settings.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.common.util.combine
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.repository.AppVersionInfoRepository
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.lib.usecase.MultihopGuideMigrationHintUseCase

class SettingsViewModel(
    deviceRepository: DeviceRepository,
    appVersionInfoRepository: AppVersionInfoRepository,
    wireguardConstraintsRepository: WireguardConstraintsRepository,
    settingsRepository: SettingsRepository,
    connectionProxy: ConnectionProxy,
    isPlayBuild: Boolean,
    multihopGuideMigrationHintUseCase: MultihopGuideMigrationHintUseCase,
) : ViewModel() {

    val uiState: StateFlow<Lc<Unit, SettingsUiState>> =
        combine(
                deviceRepository.deviceState,
                appVersionInfoRepository.versionInfo,
                wireguardConstraintsRepository.wireguardConstraints,
                settingsRepository.settingsUpdates,
                connectionProxy.tunnelState,
                multihopGuideMigrationHintUseCase(),
            ) {
                deviceState,
                versionInfo,
                wireguardConstraints,
                settings,
                tunnelState,
                showMigrationGuideNotificationDot ->
                SettingsUiState(
                        isLoggedIn = deviceState is DeviceState.LoggedIn,
                        appVersion = versionInfo.currentVersion,
                        isSupportedVersion = versionInfo.isSupported,
                        multihopMode = wireguardConstraints?.multihop ?: MultihopMode.WHEN_NEEDED,
                        isDaitaEnabled = settings?.tunnelOptions?.daitaSettings?.enabled == true,
                        splitTunnelingIsActive =
                            settings?.splitTunnelSettings?.enabled == true &&
                                tunnelState.isSecured(),
                        showMigrationGuideNotificationDot = showMigrationGuideNotificationDot,
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

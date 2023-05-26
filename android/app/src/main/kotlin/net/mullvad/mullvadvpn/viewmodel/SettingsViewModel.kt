package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import org.joda.time.DateTime

class SettingsViewModel(
    accountRepository: AccountRepository,
    deviceRepository: DeviceRepository,
    serviceConnectionManager: ServiceConnectionManager
) : ViewModel() {

    private val vmState =
        combine(
                deviceRepository.deviceState,
                accountRepository.accountExpiryState,
                serviceConnectionManager.connectionState
            ) { deviceState, expiryState, versionInfo ->
                val cachedVersionInfo = versionInfo.readyContainer()?.appVersionInfoCache
                SettingsUiState(
                    isLoggedIn = deviceState is DeviceState.LoggedIn,
                    accountExpiry = expiryState.date(),
                    appVersion = cachedVersionInfo?.version ?: "",
                    isUpdateAvailable =
                        !(cachedVersionInfo?.isSupported ?: false) ||
                            (cachedVersionInfo?.isOutdated ?: false)
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                VpnSettingsViewModelState.default()
            )

    val uiState =
        vmState.stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(),
            SettingsUiState(false, DateTime.now(), "", false)
        )
}

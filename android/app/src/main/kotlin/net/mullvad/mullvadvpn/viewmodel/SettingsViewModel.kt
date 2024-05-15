package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.lib.account.AccountRepository
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoRepository

class SettingsViewModel(
    accountRepository: AccountRepository,
    appVersionInfoRepository: AppVersionInfoRepository,
    isPlayBuild: Boolean
) : ViewModel() {

    private val vmState: StateFlow<SettingsUiState> =
        combine(accountRepository.accountState, appVersionInfoRepository.versionInfo()) {
                accountState,
                versionInfo ->
                SettingsUiState(
                    isLoggedIn = accountState is DeviceState.LoggedIn,
                    appVersion = BuildConfig.VERSION_NAME,
                    isUpdateAvailable =
                        versionInfo.let { it.isSupported.not() || it.isUpdateAvailable },
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

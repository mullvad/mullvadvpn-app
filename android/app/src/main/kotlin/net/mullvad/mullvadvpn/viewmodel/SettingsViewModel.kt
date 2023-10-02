package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager

class SettingsViewModel(
    deviceRepository: DeviceRepository,
    serviceConnectionManager: ServiceConnectionManager
) : ViewModel() {
    private val _enterTransitionEndAction = MutableSharedFlow<Unit>()

    private val vmState: StateFlow<SettingsUiState> =
        combine(deviceRepository.deviceState, serviceConnectionManager.connectionState) {
                deviceState,
                versionInfo ->
                val cachedVersionInfo = versionInfo.readyContainer()?.appVersionInfoCache
                SettingsUiState(
                    isLoggedIn = deviceState is DeviceState.LoggedIn,
                    appVersion = cachedVersionInfo?.version ?: "",
                    isUpdateAvailable =
                        cachedVersionInfo?.let { it.isSupported.not() || it.isOutdated } ?: false
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SettingsUiState(appVersion = "", isLoggedIn = false, isUpdateAvailable = false)
            )

    val uiState =
        vmState.stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(),
            SettingsUiState(appVersion = "", isLoggedIn = false, isUpdateAvailable = false)
        )

    @Suppress("konsist.ensure public properties use permitted names")
    val enterTransitionEndAction = _enterTransitionEndAction.asSharedFlow()

    fun onTransitionAnimationEnd() {
        viewModelScope.launch { _enterTransitionEndAction.emit(Unit) }
    }
}

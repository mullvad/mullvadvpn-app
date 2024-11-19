package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.ConnectError
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.WebsiteAuthToken
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.lib.shared.VpnPermissionRepository
import net.mullvad.mullvadvpn.repository.InAppNotificationController
import net.mullvad.mullvadvpn.repository.NewDeviceRepository
import net.mullvad.mullvadvpn.usecase.LastKnownLocationUseCase
import net.mullvad.mullvadvpn.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationTitleUseCase
import net.mullvad.mullvadvpn.util.combine
import net.mullvad.mullvadvpn.util.daysFromNow
import net.mullvad.mullvadvpn.util.isSuccess

@Suppress("LongParameterList")
class ConnectViewModel(
    private val accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    inAppNotificationController: InAppNotificationController,
    private val newDeviceRepository: NewDeviceRepository,
    selectedLocationTitleUseCase: SelectedLocationTitleUseCase,
    private val outOfTimeUseCase: OutOfTimeUseCase,
    private val paymentUseCase: PaymentUseCase,
    private val connectionProxy: ConnectionProxy,
    lastKnownLocationUseCase: LastKnownLocationUseCase,
    private val vpnPermissionRepository: VpnPermissionRepository,
    private val resources: Resources,
    private val isPlayBuild: Boolean,
    private val packageName: String,
) : ViewModel() {
    private val _uiSideEffect = Channel<UiSideEffect>()

    val uiSideEffect =
        merge(_uiSideEffect.receiveAsFlow(), outOfTimeEffect(), revokedDeviceEffect())

    @OptIn(FlowPreview::class)
    val uiState: StateFlow<ConnectUiState> =
        combine(
                selectedLocationTitleUseCase(),
                inAppNotificationController.notifications,
                connectionProxy.tunnelState,
                lastKnownLocationUseCase.lastKnownDisconnectedLocation,
                accountRepository.accountData,
                deviceRepository.deviceState.map { it?.displayName() },
            ) {
                selectedRelayItemTitle,
                notifications,
                tunnelState,
                lastKnownDisconnectedLocation,
                accountData,
                deviceName ->
                ConnectUiState(
                    location =
                        when (tunnelState) {
                            is TunnelState.Disconnected ->
                                tunnelState.location ?: lastKnownDisconnectedLocation
                            is TunnelState.Connecting -> tunnelState.location
                            is TunnelState.Connected -> tunnelState.location
                            is TunnelState.Disconnecting -> lastKnownDisconnectedLocation
                            is TunnelState.Error -> lastKnownDisconnectedLocation
                        },
                    selectedRelayItemTitle = selectedRelayItemTitle,
                    tunnelState = tunnelState,
                    showLocation =
                        when (tunnelState) {
                            is TunnelState.Disconnected -> true
                            is TunnelState.Disconnecting -> {
                                when (tunnelState.actionAfterDisconnect) {
                                    ActionAfterDisconnect.Nothing -> false
                                    ActionAfterDisconnect.Block -> true
                                    ActionAfterDisconnect.Reconnect -> false
                                }
                            }
                            is TunnelState.Connecting -> false
                            is TunnelState.Connected -> false
                            is TunnelState.Error -> true
                        },
                    inAppNotification = notifications.firstOrNull(),
                    deviceName = deviceName,
                    daysLeftUntilExpiry = accountData?.expiryDate?.daysFromNow(),
                    isPlayBuild = isPlayBuild,
                )
            }
            .debounce(UI_STATE_DEBOUNCE_DURATION_MILLIS)
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), ConnectUiState.INITIAL)

    init {
        viewModelScope.launch {
            if (paymentUseCase.verifyPurchases().isSuccess()) {
                accountRepository.getAccountData()
            }
        }
        viewModelScope.launch { deviceRepository.updateDevice() }
    }

    fun onDisconnectClick() {
        viewModelScope.launch {
            connectionProxy.disconnect().onLeft {
                _uiSideEffect.send(UiSideEffect.ConnectError.Generic)
            }
        }
    }

    fun onReconnectClick() {
        viewModelScope.launch {
            connectionProxy.reconnect().onLeft {
                _uiSideEffect.send(UiSideEffect.ConnectError.Generic)
            }
        }
    }

    fun onConnectClick() {
        viewModelScope.launch {
            connectionProxy.connect().onLeft { connectError ->
                when (connectError) {
                    ConnectError.NoVpnPermission -> _uiSideEffect.send(UiSideEffect.NoVpnPermission)
                    is ConnectError.Unknown -> {
                        _uiSideEffect.send(UiSideEffect.ConnectError.Generic)
                    }
                }
            }
        }
    }

    fun requestVpnPermissionResult(hasVpnPermission: Boolean) {
        viewModelScope.launch {
            if (hasVpnPermission) {
                connectionProxy.connect()
            } else {
                vpnPermissionRepository.getAlwaysOnVpnAppName()?.let {
                    _uiSideEffect.send(UiSideEffect.ConnectError.AlwaysOnVpn(it))
                } ?: _uiSideEffect.send(UiSideEffect.ConnectError.NoVpnPermission)
            }
        }
    }

    fun onCancelClick() {
        viewModelScope.launch {
            connectionProxy.disconnect().onLeft {
                _uiSideEffect.send(UiSideEffect.ConnectError.Generic)
            }
        }
    }

    fun onManageAccountClick() {
        viewModelScope.launch {
            val wwwAuthToken = accountRepository.getWebsiteAuthToken()
            _uiSideEffect.send(UiSideEffect.OpenAccountManagementPageInBrowser(wwwAuthToken))
        }
    }

    fun openAppListing() =
        viewModelScope.launch {
            val uri =
                if (isPlayBuild) {
                    resources.getString(R.string.market_uri, packageName)
                } else {
                    resources.getString(R.string.download_url)
                }
            _uiSideEffect.send(UiSideEffect.OpenUri(Uri.parse(uri)))
        }

    fun dismissNewDeviceNotification() {
        newDeviceRepository.clearNewDeviceCreatedNotification()
    }

    private fun outOfTimeEffect() =
        outOfTimeUseCase.isOutOfTime.filter { it == true }.map { UiSideEffect.OutOfTime }

    private fun revokedDeviceEffect() =
        deviceRepository.deviceState.filterIsInstance<DeviceState.Revoked>().map {
            UiSideEffect.RevokedDevice
        }

    sealed interface UiSideEffect {
        data class OpenAccountManagementPageInBrowser(val token: WebsiteAuthToken?) : UiSideEffect

        data object OutOfTime : UiSideEffect

        data class OpenUri(val uri: Uri) : UiSideEffect

        data object RevokedDevice : UiSideEffect

        data object NoVpnPermission : UiSideEffect

        sealed interface ConnectError : UiSideEffect {
            data object Generic : ConnectError

            data object NoVpnPermission : ConnectError

            data class AlwaysOnVpn(val appName: String) : ConnectError
        }
    }

    companion object {
        const val UI_STATE_DEBOUNCE_DURATION_MILLIS: Long = 200
    }
}

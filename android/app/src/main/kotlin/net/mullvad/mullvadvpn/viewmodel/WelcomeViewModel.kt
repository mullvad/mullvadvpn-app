package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_POLL_INTERVAL
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.util.UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS
import net.mullvad.mullvadvpn.util.addDebounceForUnknownState
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import org.joda.time.DateTime

@OptIn(FlowPreview::class)
class WelcomeViewModel(
    private val accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    private val serviceConnectionManager: ServiceConnectionManager,
    private val pollAccountExpiry: Boolean = true
) : ViewModel() {

    private val _viewActions = MutableSharedFlow<ViewAction>(extraBufferCapacity = 1)
    val viewActions = _viewActions.asSharedFlow()

    private val _uiState =
        serviceConnectionManager.connectionState
            .flatMapLatest { state ->
                if (state is ServiceConnectionState.ConnectedReady) {
                    flowOf(state.container)
                } else {
                    emptyFlow()
                }
            }
            .flatMapLatest { serviceConnection ->
                combine(
                    serviceConnection.connectionProxy.tunnelUiStateFlow(),
                    deviceRepository.deviceState.debounce {
                        it.addDebounceForUnknownState(UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS)
                    }
                ) { tunnelState, deviceState ->
                    WelcomeUiState(
                        tunnelState = tunnelState,
                        accountNumber = deviceState.token()?.addSpacesToAccountText()
                    )
                }
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), WelcomeUiState())
    val uiState = _uiState

    init {
        viewModelScope.launch {
            accountRepository.accountExpiryState.collectLatest { accountExpiry ->
                accountExpiry.date()?.let { expiry ->
                    val tomorrow = DateTime.now().plusHours(20)

                    if (expiry.isAfter(tomorrow)) {
                        _viewActions.tryEmit(ViewAction.OpenConnectScreen)
                    }
                }
            }
        }
        viewModelScope.launch {
            while (pollAccountExpiry) {
                accountRepository.fetchAccountExpiry()
                delay(ACCOUNT_EXPIRY_POLL_INTERVAL)
            }
        }
    }

    private fun ConnectionProxy.tunnelUiStateFlow(): Flow<TunnelState> =
        callbackFlowFromNotifier(this.onUiStateChange)

    private fun String.addSpacesToAccountText(): String {
        val length = this.length

        return if (length == 0) {
            ""
        } else {
            val numParts = (length - 1) / 4 + 1

            val parts =
                Array(numParts) { index ->
                    val startIndex = index * 4
                    val endIndex = minOf(startIndex + 4, length)

                    this.substring(startIndex, endIndex)
                }

            parts.joinToString(" ")
        }
    }

    fun onSitePaymentClick() {
        viewModelScope.launch {
            _viewActions.tryEmit(
                ViewAction.OpenAccountView(
                    serviceConnectionManager.authTokenCache()?.fetchAuthToken() ?: ""
                )
            )
        }
    }

    sealed interface ViewAction {
        data class OpenAccountView(val token: String) : ViewAction

        data object OpenConnectScreen : ViewAction
    }
}

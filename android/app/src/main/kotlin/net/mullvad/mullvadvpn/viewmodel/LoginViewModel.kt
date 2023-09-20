package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.LoginError
import net.mullvad.mullvadvpn.compose.state.LoginState
import net.mullvad.mullvadvpn.compose.state.LoginState.*
import net.mullvad.mullvadvpn.compose.state.LoginUiState
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository

private const val MINIMUM_LOADING_SPINNER_TIME_MILLIS = 500L

sealed interface LoginViewAction {
    data object NavigateToWelcome : LoginViewAction

    data object NavigateToConnect : LoginViewAction

    data class TooManyDevices(val accountToken: AccountToken) : LoginViewAction
}

class LoginViewModel(
    private val accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {
    private val _loginState = MutableStateFlow<LoginState>(Idle(null))

    private val _viewActions = MutableSharedFlow<LoginViewAction>(extraBufferCapacity = 1)
    val viewActions = _viewActions.asSharedFlow()

    private val _uiState =
        combine(
            accountRepository.accountHistory,
            _loginState,
        ) { accountHistoryState, loginState ->
            LoginUiState(accountHistoryState.accountToken()?.let(::AccountToken), loginState)
        }
    val uiState: StateFlow<LoginUiState> =
        _uiState.stateIn(viewModelScope, SharingStarted.WhileSubscribed(), LoginUiState.INITIAL)

    fun clearAccountHistory() = accountRepository.clearAccountHistory()

    fun createAccount() {
        _loginState.value = Loading.CreatingAccount
        viewModelScope.launch(dispatcher) {
            accountRepository.createAccount().mapToUiState()?.let { _loginState.value = it }
        }
    }

    fun login(accountToken: String) {
        _loginState.value = Loading.LoggingIn
        viewModelScope.launch(dispatcher) {
            // Ensure we always take at least MINIMUM_LOADING_SPINNER_TIME_MILLIS to show the
            // loading indicator
            val loginDeferred = async { accountRepository.login(accountToken) }
            delay(MINIMUM_LOADING_SPINNER_TIME_MILLIS)

            val uiState =
                when (val result = loginDeferred.await()) {
                    LoginResult.Ok -> {
                        launch {
                            delay(1000)
                            _viewActions.emit(LoginViewAction.NavigateToConnect)
                        }
                        Success
                    }
                    LoginResult.InvalidAccount -> Idle(LoginError.InvalidCredentials)
                    LoginResult.MaxDevicesReached -> {
                        // TODO this refresh process should be handled by DeviceListScreen.
                        val refreshResult =
                            deviceRepository.refreshAndAwaitDeviceListWithTimeout(
                                accountToken = accountToken,
                                shouldClearCache = true,
                                shouldOverrideCache = true,
                                timeoutMillis = 5000L
                            )

                        if (refreshResult.isAvailable()) {
                            // Navigate to device list
                            _viewActions.emit(
                                LoginViewAction.TooManyDevices(AccountToken(accountToken))
                            )
                            return@launch
                        } else {
                            // Failed to fetch devices list
                            Idle(LoginError.Unknown(result.toString()))
                        }
                    }
                    else -> Idle(LoginError.Unknown(result.toString()))
                }
            _loginState.update { uiState }
        }
    }

    private suspend fun AccountCreationResult.mapToUiState(): LoginState? {
        return if (this is AccountCreationResult.Success) {
            _viewActions.emit(LoginViewAction.NavigateToWelcome)
            null
        } else {
            Idle(LoginError.UnableToCreateAccount)
        }
    }
}

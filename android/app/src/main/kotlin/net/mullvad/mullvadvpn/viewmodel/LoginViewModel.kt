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

sealed interface LoginUiSideEffect {
    data object NavigateToWelcome : LoginUiSideEffect

    data object NavigateToConnect : LoginUiSideEffect

    data class TooManyDevices(val accountToken: AccountToken) : LoginUiSideEffect
}

class LoginViewModel(
    private val accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {
    private val _loginState = MutableStateFlow(LoginUiState.INITIAL.loginState)
    private val _loginInput = MutableStateFlow(LoginUiState.INITIAL.accountNumberInput)

    private val _uiSideEffect = MutableSharedFlow<LoginUiSideEffect>(extraBufferCapacity = 1)
    val uiSideEffect = _uiSideEffect.asSharedFlow()

    private val _uiState =
        combine(
            _loginInput,
            accountRepository.accountHistory,
            _loginState,
        ) { loginInput, accountHistoryState, loginState ->
            LoginUiState(
                loginInput,
                accountHistoryState.accountToken()?.let(::AccountToken),
                loginState
            )
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
                            _uiSideEffect.emit(LoginUiSideEffect.NavigateToConnect)
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
                            _uiSideEffect.emit(
                                LoginUiSideEffect.TooManyDevices(AccountToken(accountToken))
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

    fun onAccountNumberChange(accountNumber: String) {
        _loginInput.value = accountNumber.filter { it.isDigit() }
        // If there is an error, clear it
        _loginState.update { if (it is Idle) Idle() else it }
    }

    private suspend fun AccountCreationResult.mapToUiState(): LoginState? {
        return if (this is AccountCreationResult.Success) {
            _uiSideEffect.emit(LoginUiSideEffect.NavigateToWelcome)
            null
        } else {
            Idle(LoginError.UnableToCreateAccount)
        }
    }
}

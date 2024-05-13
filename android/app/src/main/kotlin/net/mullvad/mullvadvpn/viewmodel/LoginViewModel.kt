package net.mullvad.mullvadvpn.viewmodel

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.LoginError
import net.mullvad.mullvadvpn.compose.state.LoginState
import net.mullvad.mullvadvpn.compose.state.LoginState.Idle
import net.mullvad.mullvadvpn.compose.state.LoginState.Loading
import net.mullvad.mullvadvpn.compose.state.LoginState.Success
import net.mullvad.mullvadvpn.compose.state.LoginUiState
import net.mullvad.mullvadvpn.lib.account.AccountRepository
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.model.LoginAccountError
import net.mullvad.mullvadvpn.usecase.ConnectivityUseCase
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.util.getOrDefault

private const val MINIMUM_LOADING_SPINNER_TIME_MILLIS = 500L

sealed interface LoginUiSideEffect {
    data object NavigateToWelcome : LoginUiSideEffect

    data object NavigateToConnect : LoginUiSideEffect

    data object NavigateToOutOfTime : LoginUiSideEffect

    data class TooManyDevices(val accountToken: AccountToken) : LoginUiSideEffect
}

class LoginViewModel(
    private val accountRepository: AccountRepository,
    private val newDeviceNotificationUseCase: NewDeviceNotificationUseCase,
    private val connectivityUseCase: ConnectivityUseCase,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {
    private val _loginState = MutableStateFlow(LoginUiState.INITIAL.loginState)
    private val _loginInput = MutableStateFlow(LoginUiState.INITIAL.accountNumberInput)

    private val _uiSideEffect = Channel<LoginUiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _mutableAccountHistory: MutableStateFlow<AccountToken?> = MutableStateFlow(null)

    private val _uiState =
        combine(
            _loginInput,
            _mutableAccountHistory,
            _loginState,
        ) { loginInput, historyAccountToken, loginState ->
            LoginUiState(loginInput, historyAccountToken, loginState)
        }

    val uiState: StateFlow<LoginUiState> =
        _uiState
            .onStart {
                viewModelScope.launch {
                    _mutableAccountHistory.update { accountRepository.fetchAccountHistory() }
                }
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), LoginUiState.INITIAL)

    fun clearAccountHistory() =
        viewModelScope.launch {
            accountRepository.clearAccountHistory()
            _mutableAccountHistory.update { null }
            _mutableAccountHistory.update { accountRepository.fetchAccountHistory() }
        }

    fun createAccount() {
        _loginState.value = Loading.CreatingAccount
        viewModelScope.launch(dispatcher) {
            accountRepository
                .createAccount()
                .fold(
                    { _loginState.value = Idle(LoginError.UnableToCreateAccount) },
                    { _uiSideEffect.send(LoginUiSideEffect.NavigateToWelcome) }
                )
        }
    }

    fun login(accountToken: String) {
        if (!isInternetAvailable()) {
            _loginState.value = Idle(LoginError.NoInternetConnection)
            return
        }
        _loginState.value = Loading.LoggingIn
        viewModelScope.launch(dispatcher) {
            // Ensure we always take at least MINIMUM_LOADING_SPINNER_TIME_MILLIS to show the
            // loading indicator
            val result = async { accountRepository.login(AccountToken(accountToken)) }

            delay(MINIMUM_LOADING_SPINNER_TIME_MILLIS)

            val uiState =
                result
                    .await()
                    .fold(
                        { it.toUiState() },
                        {
                            newDeviceNotificationUseCase.newDeviceCreated()
                            launch {
                                val isOutOfTime = isOutOfTime()
                                if (isOutOfTime) {
                                    _uiSideEffect.send(LoginUiSideEffect.NavigateToOutOfTime)
                                } else {
                                    _uiSideEffect.send(LoginUiSideEffect.NavigateToConnect)
                                }
                            }
                            Success
                        }
                    )

            _loginState.update { uiState }
        }
    }

    private suspend fun isOutOfTime(): Boolean = coroutineScope {
        Log.d("LoginViewModel", "isOutOfTime")
        val isOutOfTimeDeferred = async {
            accountRepository.accountData.filterNotNull().map { it.expiryDate.isBeforeNow }.first()
        }
        Log.d("LoginViewModel", "isOutOfTimeDeferred: $isOutOfTimeDeferred")
        delay(1000)
        Log.d("LoginViewModel", "finished waiting")
        val result = isOutOfTimeDeferred.getOrDefault(false)
        Log.d("LoginViewModel", "Result: $result")
        result
    }

    fun onAccountNumberChange(accountNumber: String) {
        _loginInput.value = accountNumber.filter { it.isDigit() }
        // If there is an error, clear it
        _loginState.update { if (it is Idle) Idle() else it }
    }

    private suspend fun LoginAccountError.toUiState(): LoginState =
        when (this) {
            LoginAccountError.InvalidAccount -> Idle(LoginError.InvalidCredentials)
            is LoginAccountError.MaxDevicesReached ->
                Idle().also { _uiSideEffect.send(LoginUiSideEffect.TooManyDevices(accountToken)) }
            LoginAccountError.RpcError,
            is LoginAccountError.Unknown -> Idle(LoginError.Unknown(this.toString()))
        }

    private fun isInternetAvailable(): Boolean {
        return connectivityUseCase.isInternetAvailable()
    }
}

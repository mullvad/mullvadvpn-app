package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.mapNotNull
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
import net.mullvad.mullvadvpn.lib.common.util.isBeforeNowInstant
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.LoginAccountError
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.repository.NewDeviceRepository
import net.mullvad.mullvadvpn.usecase.InternetAvailableUseCase
import net.mullvad.mullvadvpn.util.delayAtLeast
import net.mullvad.mullvadvpn.util.getOrDefault

private const val MINIMUM_LOADING_SPINNER_TIME_MILLIS = 500L

sealed interface LoginUiSideEffect {
    data object NavigateToWelcome : LoginUiSideEffect

    data object NavigateToConnect : LoginUiSideEffect

    data object NavigateToOutOfTime : LoginUiSideEffect

    data object NavigateToCreateAccountConfirmation : LoginUiSideEffect

    data class TooManyDevices(val accountNumber: AccountNumber) : LoginUiSideEffect

    data object GenericError : LoginUiSideEffect
}

class LoginViewModel(
    private val accountRepository: AccountRepository,
    private val newDeviceRepository: NewDeviceRepository,
    private val internetAvailableUseCase: InternetAvailableUseCase,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {
    private val _loginState = MutableStateFlow(LoginUiState.INITIAL.loginState)
    private val _loginInput = MutableStateFlow(LoginUiState.INITIAL.accountNumberInput)

    private val _uiSideEffect = Channel<LoginUiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val _uiState =
        combine(_loginInput, accountRepository.accountHistory, _loginState) {
            loginInput,
            historyAccountNumber,
            loginState ->
            LoginUiState(loginInput, historyAccountNumber, loginState)
        }

    val uiState: StateFlow<LoginUiState> =
        _uiState
            .onStart { viewModelScope.launch { accountRepository.fetchAccountHistory() } }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), LoginUiState.INITIAL)

    fun clearAccountHistory() =
        viewModelScope.launch {
            accountRepository.clearAccountHistory().onLeft {
                _uiSideEffect.send(LoginUiSideEffect.GenericError)
            }
        }

    fun onCreateAccountClick() {
        if (hasPreviouslyCreatedAccount()) {
            viewModelScope.launch {
                _uiSideEffect.send(LoginUiSideEffect.NavigateToCreateAccountConfirmation)
            }
        } else {
            createAccount()
        }
    }

    fun onCreateAccountConfirmed() {
        createAccount()
    }

    private fun createAccount() {
        _loginState.value = Loading.CreatingAccount
        viewModelScope.launch(dispatcher) {
            accountRepository
                .createAccount()
                .fold(
                    { _loginState.value = Idle(LoginError.UnableToCreateAccount) },
                    { _uiSideEffect.send(LoginUiSideEffect.NavigateToWelcome) },
                )
        }
    }

    fun login(accountNumber: String) {
        if (!isInternetAvailable()) {
            _loginState.value = Idle(LoginError.NoInternetConnection)
            return
        }
        _loginState.value = Loading.LoggingIn
        viewModelScope.launch(dispatcher) {
            val uiState =
                // Ensure we always take at least MINIMUM_LOADING_SPINNER_TIME_MILLIS to show the
                // loading indicator
                delayAtLeast(MINIMUM_LOADING_SPINNER_TIME_MILLIS) {
                        accountRepository.login(AccountNumber(accountNumber))
                    }
                    .fold(
                        { it.toUiState() },
                        {
                            onSuccessfulLogin()
                            Success
                        },
                    )

            _loginState.update { uiState }
        }
    }

    private fun onSuccessfulLogin() {
        newDeviceRepository.newDeviceCreated()

        viewModelScope.launch(dispatcher) {
            // Find if user is out of time
            val isOutOfTimeDeferred = async {
                accountRepository.accountData
                    .mapNotNull { it?.expiryDate?.isBeforeNowInstant() }
                    .first()
            }

            // Always show successful login for some time.
            delay(SHOW_SUCCESSFUL_LOGIN_MILLIS)

            // Get the result of isOutOfTime or assume not out of time
            val isOutOfTime = isOutOfTimeDeferred.getOrDefault(false)

            if (isOutOfTime) {
                _uiSideEffect.send(LoginUiSideEffect.NavigateToOutOfTime)
            } else {
                _uiSideEffect.send(LoginUiSideEffect.NavigateToConnect)
            }
        }
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
                Idle().also { _uiSideEffect.send(LoginUiSideEffect.TooManyDevices(accountNumber)) }
            LoginAccountError.RpcError,
            is LoginAccountError.Unknown -> Idle(LoginError.Unknown(this.toString()))
        }

    private fun isInternetAvailable(): Boolean {
        return internetAvailableUseCase()
    }

    private fun hasPreviouslyCreatedAccount(): Boolean = uiState.value.lastUsedAccount != null

    companion object {
        private const val SHOW_SUCCESSFUL_LOGIN_MILLIS = 1000L
    }
}

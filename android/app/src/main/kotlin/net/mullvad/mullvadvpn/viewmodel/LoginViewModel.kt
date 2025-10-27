package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.LoginState
import net.mullvad.mullvadvpn.compose.state.LoginState.Idle
import net.mullvad.mullvadvpn.compose.state.LoginState.Loading
import net.mullvad.mullvadvpn.compose.state.LoginState.Success
import net.mullvad.mullvadvpn.compose.state.LoginUiState
import net.mullvad.mullvadvpn.compose.state.LoginUiStateError
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.isBeforeNowInstant
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.CreateAccountError
import net.mullvad.mullvadvpn.lib.model.LoginAccountError
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.NewDeviceRepository
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.AccountExpiryNotificationProvider
import net.mullvad.mullvadvpn.usecase.InternetAvailableUseCase
import net.mullvad.mullvadvpn.usecase.ScheduleNotificationAlarmUseCase
import net.mullvad.mullvadvpn.util.delayAtLeast
import net.mullvad.mullvadvpn.util.getOrDefault
import net.mullvad.mullvadvpn.viewmodel.LoginUiSideEffect.NavigateToWelcome
import net.mullvad.mullvadvpn.viewmodel.LoginUiSideEffect.TooManyDevices

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
    private val scheduleNotificationAlarmUseCase: ScheduleNotificationAlarmUseCase,
    private val accountExpiryNotificationProvider: AccountExpiryNotificationProvider,
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
            .onStart {
                viewModelScope.launch {
                    accountRepository.fetchAccountHistory()
                    accountExpiryNotificationProvider.cancelNotification()
                    scheduleNotificationAlarmUseCase(accountExpiry = null)
                }
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                LoginUiState.INITIAL,
            )

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
                    { _loginState.value = it.toUiState() },
                    { _uiSideEffect.send(NavigateToWelcome) },
                )
        }
    }

    fun login(accountNumber: String) {
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
            LoginAccountError.InvalidAccount ->
                Idle(LoginUiStateError.LoginError.InvalidCredentials)
            is LoginAccountError.MaxDevicesReached ->
                Idle().also { _uiSideEffect.send(TooManyDevices(accountNumber)) }
            is LoginAccountError.InvalidInput ->
                Idle(LoginUiStateError.LoginError.InvalidInput(accountNumber))
            LoginAccountError.Timeout,
            LoginAccountError.ApiUnreachable ->
                if (isInternetAvailable()) {
                    Idle(LoginUiStateError.LoginError.ApiUnreachable)
                } else {
                    Idle(LoginUiStateError.LoginError.NoInternetConnection)
                }
            LoginAccountError.TooManyAttempts -> Idle(LoginUiStateError.LoginError.TooManyAttempts)
            is LoginAccountError.Unknown ->
                Idle(LoginUiStateError.LoginError.Unknown(this.toString())).also {
                    Logger.w("Login failed with error: $this", error)
                }
        }

    private fun CreateAccountError.toUiState(): LoginState =
        when (this) {
            CreateAccountError.ApiUnreachable,
            CreateAccountError.TimeOut ->
                if (isInternetAvailable()) {
                    Idle(LoginUiStateError.CreateAccountError.ApiUnreachable)
                } else {
                    Idle(LoginUiStateError.CreateAccountError.NoInternetConnection)
                }
            CreateAccountError.TooManyAttempts ->
                Idle(LoginUiStateError.CreateAccountError.TooManyAttempts)
            is CreateAccountError.Unknown ->
                Idle(LoginUiStateError.CreateAccountError.Unknown).also {
                    Logger.w("Create account failed with error: $this", error)
                }
        }

    private fun isInternetAvailable(): Boolean {
        return internetAvailableUseCase()
    }

    private fun hasPreviouslyCreatedAccount(): Boolean = uiState.value.lastUsedAccount != null

    companion object {
        private const val SHOW_SUCCESSFUL_LOGIN_MILLIS = 1000L
    }
}

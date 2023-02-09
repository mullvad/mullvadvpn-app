package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository

class LoginViewModel(
    private val accountRepository: AccountRepository,
    private val deviceRepository: DeviceRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {
    private val _uiState = MutableStateFlow<LoginUiState>(LoginUiState.Default)
    val uiState: StateFlow<LoginUiState> = _uiState

    val accountHistory = accountRepository.accountHistoryEvents

    sealed class LoginUiState {
        object Default : LoginUiState()
        object Loading : LoginUiState()
        data class Success(
            val isOutOfTime: Boolean
        ) : LoginUiState()

        object CreatingAccount : LoginUiState()
        object AccountCreated : LoginUiState()
        object UnableToCreateAccountError : LoginUiState()
        object InvalidAccountError : LoginUiState()
        data class TooManyDevicesError(val accountToken: String) : LoginUiState()
        object TooManyDevicesMissingListError : LoginUiState()
        data class OtherError(val errorMessage: String) : LoginUiState()

        fun isLoading(): Boolean {
            return this is Loading
        }
    }

    fun clearAccountHistory() = accountRepository.clearAccountHistory()

    fun clearState() {
        _uiState.value = LoginUiState.Default
    }

    fun createAccount() {
        _uiState.value = LoginUiState.CreatingAccount
        viewModelScope.launch(dispatcher) {
            _uiState.value = accountRepository.accountCreationEvents
                .onStart { accountRepository.createAccount() }
                .first()
                .mapToUiState()
        }
    }

    fun login(accountToken: String) {
        _uiState.value = LoginUiState.Loading
        viewModelScope.launch(dispatcher) {
            _uiState.value = accountRepository.loginEvents
                .onStart { accountRepository.login(accountToken) }
                .map { it.result.mapToUiState(accountToken) }
                .first()
        }
    }

    private fun AccountCreationResult.mapToUiState(): LoginUiState {
        return if (this is AccountCreationResult.Success) {
            LoginUiState.AccountCreated
        } else {
            LoginUiState.UnableToCreateAccountError
        }
    }

    private suspend fun LoginResult.mapToUiState(accountToken: String): LoginUiState {
        return when (this) {
            LoginResult.Ok -> LoginUiState.Success(false)
            LoginResult.InvalidAccount -> LoginUiState.InvalidAccountError
            LoginResult.MaxDevicesReached -> {
                val refreshResult = deviceRepository.refreshAndAwaitDeviceListWithTimeout(
                    accountToken = accountToken,
                    shouldClearCache = true,
                    shouldOverrideCache = true,
                    timeoutMillis = 5000L
                )

                if (refreshResult.isAvailable()) {
                    LoginUiState.TooManyDevicesError(accountToken)
                } else {
                    LoginUiState.TooManyDevicesMissingListError
                }
            }
            else -> LoginUiState.OtherError(errorMessage = this.toString())
        }
    }
}

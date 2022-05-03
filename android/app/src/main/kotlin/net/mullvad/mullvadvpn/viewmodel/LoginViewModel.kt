package net.mullvad.mullvadvpn.viewmodel

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountHistory
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountCache

class LoginViewModel(
    application: Application
) : AndroidViewModel(application) {
    private val _uiState = MutableStateFlow<LoginUiState>(LoginUiState.Default)
    val uiState: StateFlow<LoginUiState> = _uiState

    private val _accountHistory = MutableStateFlow<AccountHistory>(AccountHistory.Missing)
    val accountHistory: StateFlow<AccountHistory> = _accountHistory

    private var accountCache: AccountCache? = null

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
        object TooManyDevicesError : LoginUiState()
        data class OtherError(val errorMessage: String) : LoginUiState()
    }

    // Ensures the view model has an up-to-date instance of account cache. This is an intermediate
    // solution due to limitations in the current app architecture.
    fun updateAccountCacheInstance(newAccountCache: AccountCache?) {
        accountCache = newAccountCache?.apply {
            viewModelScope.launch {
                accountHistoryEvents.collect {
                    _accountHistory.value = it
                }
            }

            fetchAccountHistory()
        }
    }

    fun clearAccountHistory() {
        accountCache?.clearAccountHistory()
    }

    fun createAccount() {
        accountCache?.apply {
            _uiState.value = LoginUiState.CreatingAccount

            viewModelScope.launch {
                _uiState.value = accountCreationEvents.first().mapToUiState()
            }

            createNewAccount()
        }
    }

    fun login(accountToken: String) {
        accountCache?.apply {
            _uiState.value = LoginUiState.Loading

            viewModelScope.launch {
                _uiState.value = loginEvents.first().result.mapToUiState()
            }

            login(accountToken)
        }
    }

    private fun AccountCreationResult.mapToUiState(): LoginUiState {
        return when (this) {
            is AccountCreationResult.Success -> LoginUiState.AccountCreated
            AccountCreationResult.Failure -> LoginUiState.UnableToCreateAccountError
        }
    }

    private fun LoginResult.mapToUiState(): LoginUiState {
        return when (this) {
            LoginResult.Ok -> LoginUiState.Success(false)
            LoginResult.InvalidAccount -> LoginUiState.InvalidAccountError
            LoginResult.MaxDevicesReached -> LoginUiState.TooManyDevicesError
            else -> LoginUiState.OtherError(errorMessage = this.toString())
        }
    }

    class Factory(val application: Application) :
        ViewModelProvider.Factory {
        override fun <T : ViewModel> create(modelClass: Class<T>): T {
            return LoginViewModel(application) as T
        }
    }
}

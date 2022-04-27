package net.mullvad.mullvadvpn.viewmodel

import android.app.Application
import androidx.annotation.RestrictTo
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountCache

class LoginViewModel(
    application: Application
) : AndroidViewModel(application) {
    private val _uiState = MutableStateFlow<LoginUiState>(LoginUiState.Default)
    private val _accountHistory = MutableStateFlow<String?>(null)
    val uiState: StateFlow<LoginUiState> = _uiState
    val accountHistory: StateFlow<String?> = _accountHistory

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
        accountCache?.unsubscribe()
        accountCache = newAccountCache?.apply { subscribe() }
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

    @RestrictTo(RestrictTo.Scope.TESTS)
    public override fun onCleared() {
        accountCache?.unsubscribe()
    }

    private fun AccountCache.subscribe() {
        onAccountHistoryChange.subscribe(this) { history ->
            _accountHistory.value = history
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

    private fun AccountCache.unsubscribe() {
        onAccountHistoryChange.unsubscribe(this)
        onLoginStatusChange.unsubscribe(this)
    }

    class Factory(val application: Application) :
        ViewModelProvider.Factory {
        override fun <T : ViewModel> create(modelClass: Class<T>): T {
            return LoginViewModel(application) as T
        }
    }
}

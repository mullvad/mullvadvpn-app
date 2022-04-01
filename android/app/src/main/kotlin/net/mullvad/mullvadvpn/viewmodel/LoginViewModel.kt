package net.mullvad.mullvadvpn.viewmodel

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
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
        _uiState.value = LoginUiState.CreatingAccount
        accountCache?.createNewAccount()
    }

    fun login(accountToken: String) {
        _uiState.value = LoginUiState.Loading
        accountCache?.login(accountToken)
    }

    override fun onCleared() {
        accountCache?.onAccountHistoryChange?.unsubscribe(this)
        accountCache?.onLoginStatusChange?.unsubscribe(this)
    }

    private fun AccountCache.subscribe() {
        onAccountHistoryChange.subscribe(this) { history ->
            _accountHistory.value = history
        }

        onLoginStatusChange.subscribe(this, startWithLatestEvent = false) { status ->
            _uiState.value = when {
                status == null -> {
                    LoginUiState.Default
                }
                status.isNewAccount -> {
                    LoginUiState.AccountCreated
                }
                else -> {
                    when (status.loginResult) {
                        LoginResult.Ok -> LoginUiState.Success(false)
                        LoginResult.InvalidAccount -> LoginUiState.InvalidAccountError
                        LoginResult.MaxDevicesReached -> LoginUiState.TooManyDevicesError
                        else -> LoginUiState.OtherError(
                            errorMessage = status.loginResult?.toString() ?: ""
                        )
                    }
                }
            }
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

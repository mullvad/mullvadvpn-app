package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.model.AccountCreationResult
import net.mullvad.mullvadvpn.model.AccountHistory
import net.mullvad.mullvadvpn.model.LoginResult
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState

class LoginViewModel(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {
    private val _uiState = MutableStateFlow<LoginUiState>(LoginUiState.Default)
    val uiState: StateFlow<LoginUiState> = _uiState

    private val accountCache: AccountCache?
        get() {
            return serviceConnectionManager.connectionState.value.readyContainer()?.accountCache
        }

    val accountHistory = serviceConnectionManager.connectionState
        .flatMapLatest { state ->
            if (state is ServiceConnectionState.ConnectedReady) {
                state.container.accountCache.accountHistoryEvents
                    .onStart {
                        state.container.accountCache.fetchAccountHistory()
                    }
            } else {
                emptyFlow()
            }
        }
        .stateIn(
            scope = CoroutineScope(dispatcher),
            started = SharingStarted.WhileSubscribed(),
            initialValue = AccountHistory.Missing
        )

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

    fun clearAccountHistory() {
        accountCache.tryPerformAction(
            errorMessageIfAccountCacheNotAvailable = SERVICE_NOT_CONNECTED_ERROR_MESSAGE
        ) { cache ->
            cache.clearAccountHistory()
        }
    }

    fun createAccount() {
        accountCache.tryPerformAction(
            errorMessageIfAccountCacheNotAvailable = SERVICE_NOT_CONNECTED_ERROR_MESSAGE
        ) { cache ->
            _uiState.value = LoginUiState.CreatingAccount
            viewModelScope.launch(dispatcher) {
                _uiState.value = cache.accountCreationEvents
                    .onStart { cache.createNewAccount() }
                    .first()
                    .mapToUiState()
            }
        }
    }

    fun login(accountToken: String) {
        accountCache.tryPerformAction(
            errorMessageIfAccountCacheNotAvailable = SERVICE_NOT_CONNECTED_ERROR_MESSAGE
        ) { cache ->
            _uiState.value = LoginUiState.Loading
            viewModelScope.launch(dispatcher) {
                _uiState.value = cache.loginEvents
                    .onStart { cache.login(accountToken) }
                    .map { it.result.mapToUiState() }
                    .first()
            }
        }
    }

    private fun AccountCache?.tryPerformAction(
        errorMessageIfAccountCacheNotAvailable: String,
        action: (AccountCache) -> Unit
    ) {
        if (this != null) {
            action(this)
        } else {
            _uiState.value = LoginUiState.OtherError(errorMessageIfAccountCacheNotAvailable)
        }
    }

    private fun AccountCreationResult.mapToUiState(): LoginUiState {
        return if (this is AccountCreationResult.Success) {
            LoginUiState.AccountCreated
        } else {
            LoginUiState.UnableToCreateAccountError
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

    companion object {
        private const val SERVICE_NOT_CONNECTED_ERROR_MESSAGE = "Not connected to service!"
    }
}

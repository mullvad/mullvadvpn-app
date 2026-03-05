package net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountconfirmation

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.DeleteAccountError
import net.mullvad.mullvadvpn.lib.repository.AccountRepository

class DeleteAccountConfirmationViewModel(val accountRepository: AccountRepository) : ViewModel() {

    private val _uiSideEffect = Channel<DeleteAccountConfirmationUiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()
    private val accountInput = MutableStateFlow("")
    private val deleteError = MutableStateFlow<DeleteAccountError?>(null)
    private val isLoading = MutableStateFlow(false)

    val uiState: StateFlow<Lc<Unit, DeleteAccountConfirmationUiState>> =
        combine(
                combine(accountInput, accountRepository.accountData.filterNotNull()) {
                        input,
                        account ->
                        input == account.accountNumber.value
                    }
                    .distinctUntilChanged(),
                isLoading,
                deleteError,
            ) { hasEnterCorrectInput, isLoading, error ->
                Lc.Content(
                    DeleteAccountConfirmationUiState(
                        isLoading = isLoading,
                        hasEnterCorrectInput,
                        deleteAccountError = error,
                    )
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    fun deleteAccount() =
        viewModelScope.launch {
            isLoading.value = true
            accountRepository
                .deleteAccount()
                .fold(
                    { deleteError.value = it },
                    { _uiSideEffect.send(DeleteAccountConfirmationUiSideEffect.NavigateToLogin) },
                )
            isLoading.value = false
        }

    fun onAccountInputChanged(input: String) {
        deleteError.value = null
        accountInput.value = input
    }
}

// 2731114520706402
data class DeleteAccountConfirmationUiState(
    val isLoading: Boolean = false,
    val hasConfirmedAccount: Boolean = false,
    val deleteAccountError: DeleteAccountError? = null,
)

sealed interface DeleteAccountConfirmationUiSideEffect {
    object NavigateToLogin : DeleteAccountConfirmationUiSideEffect
}

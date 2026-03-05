package net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountconfirmation

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.DeleteAccountError
import net.mullvad.mullvadvpn.lib.repository.AccountRepository

class DeleteAccountConfirmationViewModel(val accountRepository: AccountRepository) : ViewModel() {

    private val accountInput = MutableStateFlow("")
    private val isLoading = MutableStateFlow(false)

    val uiState: StateFlow<Lc<Unit, DeleteAccountConfirmationUiState>> =
        combine(accountInput, accountRepository.accountData.filterNotNull(), isLoading) {
                input,
                account,
                loading ->
                Lc.Content(
                    DeleteAccountConfirmationUiState(
                        isLoading = loading,
                        hasConfirmedAccount = input == account.accountNumber.value,
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
            accountRepository.accountData.value
            accountRepository.deleteAccount().onLeft {}
        }

    fun onAccountInputChanged(input: String) {
        accountInput.value = input
    }
}

data class DeleteAccountConfirmationUiState(
    val isLoading: Boolean = false,
    val hasConfirmedAccount: Boolean = false,
    val deleteAccountError: DeleteAccountError? = null,
)

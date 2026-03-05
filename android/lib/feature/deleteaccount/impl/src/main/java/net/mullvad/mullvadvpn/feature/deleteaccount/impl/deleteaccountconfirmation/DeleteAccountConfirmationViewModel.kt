package net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountconfirmation

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.repository.AccountRepository

class DeleteAccountConfirmationViewModel(val accountRepository: AccountRepository) : ViewModel() {

    private val _uiState = MutableStateFlow(DeleteAccountConfirmationUiState())
    val uiState = _uiState.asStateFlow()

    fun deleteAccount() =
        viewModelScope.launch {
            accountRepository.accountData.value
            accountRepository.deleteAccount().onLeft {}
        }
}

data class DeleteAccountConfirmationUiState(
    val isLoading: Boolean = false,
    val hasConfirmedAccount: Boolean = false,
)

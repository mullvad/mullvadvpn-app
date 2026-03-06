package net.mullvad.mullvadvpn.feature.deleteaccount.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.daysLeft
import net.mullvad.mullvadvpn.lib.repository.AccountRepository

class DeleteAccountViewModel(accountRepository: AccountRepository) : ViewModel() {
    val uiState: StateFlow<Lc<Unit, DeleteAccountUiState>> =
        accountRepository.accountData
            .filterNotNull()
            .map { accountData ->
                Lc.Content(
                    DeleteAccountUiState(daysLeft = accountData.expiryDate.daysLeft()?.toInt())
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )
}

data class DeleteAccountUiState(val daysLeft: Int?)

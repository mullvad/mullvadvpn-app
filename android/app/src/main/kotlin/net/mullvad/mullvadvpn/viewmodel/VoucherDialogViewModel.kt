package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.VoucherDialogState
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.constant.VOUCHER_LENGTH
import net.mullvad.mullvadvpn.lib.account.VoucherRepository
import net.mullvad.mullvadvpn.model.RedeemVoucherError
import net.mullvad.mullvadvpn.util.VoucherRegexHelper

class VoucherDialogViewModel(
    private val voucherRepository: VoucherRepository
) : ViewModel() {

    private val vmState = MutableStateFlow<VoucherDialogState>(VoucherDialogState.Default)
    private val voucherInput = MutableStateFlow("")

    val uiState =
        combine(vmState, voucherInput) { state, input ->
                VoucherDialogUiState(voucherInput = input, voucherState = state)
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), VoucherDialogUiState.INITIAL)

    fun onRedeem(voucherCode: String) {
        vmState.update { VoucherDialogState.Verifying }
        viewModelScope.launch {
            voucherRepository
                .submitVoucher(voucherCode)
                .fold(
                    { error -> setError(error) },
                    { success -> handleAddedTime(success.timeAdded) }
                )
        }
    }

    fun onVoucherInputChange(voucherString: String) {
        // Remove any errors when the user starts typing again
        vmState.update { VoucherDialogState.Default }
        if (VoucherRegexHelper.validate(voucherString)) {
            val trimmedVoucher = VoucherRegexHelper.trim(voucherString)
            voucherInput.value =
                trimmedVoucher
                    .substring(0, Integer.min(VOUCHER_LENGTH, trimmedVoucher.length))
                    .uppercase()
        }
    }

    private fun handleAddedTime(timeAdded: Long) {
        viewModelScope.launch { vmState.update { VoucherDialogState.Success(timeAdded) } }
    }

    private fun setError(error: RedeemVoucherError) {
        viewModelScope.launch { vmState.update { VoucherDialogState.Error(error) } }
    }
}

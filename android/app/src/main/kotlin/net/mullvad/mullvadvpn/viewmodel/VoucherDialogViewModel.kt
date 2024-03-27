package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.state.VoucherDialogState
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.constant.VOUCHER_LENGTH
import net.mullvad.mullvadvpn.model.VoucherSubmissionError
import net.mullvad.mullvadvpn.util.VoucherRegexHelper

class VoucherDialogViewModel(private val resources: Resources) : ViewModel() {

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
            TODO()
            //            when (val result =
            // serviceConnectionManager.voucherRedeemer()?.submit(voucherCode)) {
            //                is VoucherSubmissionResult.Ok ->
            // handleAddedTime(result.submission.timeAdded)
            //                is VoucherSubmissionResult.Error -> setError(result.error)
            //                null -> vmState.update { VoucherDialogState.Default }
            //            }
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

    private fun setError(error: VoucherSubmissionError) {
        viewModelScope.launch {
            val message =
                resources.getString(
                    when (error) {
                        VoucherSubmissionError.InvalidVoucher -> R.string.invalid_voucher
                        VoucherSubmissionError.VoucherAlreadyUsed -> R.string.voucher_already_used
                        VoucherSubmissionError.RpcError,
                        VoucherSubmissionError.OtherError -> R.string.error_occurred
                    }
                )
            vmState.update { VoucherDialogState.Error(message) }
        }
    }
}

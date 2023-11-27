package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.state.VoucherDialogState
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.constant.VOUCHER_LENGTH
import net.mullvad.mullvadvpn.model.VoucherSubmissionError
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.voucherRedeemer
import net.mullvad.mullvadvpn.util.VoucherRegexHelper

class VoucherDialogViewModel(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val resources: Resources
) : ViewModel() {

    private val vmState = MutableStateFlow<VoucherDialogState>(VoucherDialogState.Default)
    private val voucherInput = MutableStateFlow("")

    private val _shared: SharedFlow<ServiceConnectionContainer> =
        serviceConnectionManager.connectionState
            .flatMapLatest { state ->
                if (state is ServiceConnectionState.ConnectedReady) {
                    flowOf(state.container)
                } else {
                    emptyFlow()
                }
            }
            .shareIn(viewModelScope, SharingStarted.WhileSubscribed())

    var uiState =
        _shared
            .flatMapLatest {
                combine(vmState, voucherInput) { state, input ->
                    VoucherDialogUiState(voucherInput = input, voucherViewModelState = state)
                }
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), VoucherDialogUiState.INITIAL)

    fun onRedeem(voucherCode: String) {
        vmState.update { VoucherDialogState.Verifying }
        viewModelScope.launch {
            when (val result = serviceConnectionManager.voucherRedeemer()?.submit(voucherCode)) {
                is VoucherSubmissionResult.Ok -> handleAddedTime(result.submission.timeAdded)
                is VoucherSubmissionResult.Error -> setError(result.error)
                else -> {
                    vmState.update { VoucherDialogState.Default }
                }
            }
        }
    }

    fun onVoucherInputChange(voucherString: String) {
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
                        else -> R.string.error_occurred
                    }
                )
            vmState.update { VoucherDialogState.Error(message) }
        }
    }
}

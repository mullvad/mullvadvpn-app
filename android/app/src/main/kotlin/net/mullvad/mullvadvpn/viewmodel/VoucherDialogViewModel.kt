package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.model.VoucherSubmissionError
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.VoucherRedeemer
import org.joda.time.DateTimeConstants.SECONDS_PER_DAY

class VoucherDialogViewModel(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val resources: Resources
) : ViewModel() {
    private val vmState = MutableStateFlow<VoucherDialogUiState>(VoucherDialogUiState.Default)

    private var voucherRedeemer: VoucherRedeemer? =
        serviceConnectionManager.connectionState.value.readyContainer()?.voucherRedeemer

    var uiState =
        vmState
            .asStateFlow()
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), VoucherDialogUiState.Default)

    fun onRedeem(voucherCode: String) {
        vmState.update { VoucherDialogUiState.Verifying }
        viewModelScope.launch {
            when (val result = voucherRedeemer?.submit(voucherCode)) {
                is VoucherSubmissionResult.Ok -> handleAddedTime(result.submission.timeAdded)
                is VoucherSubmissionResult.Error -> setError(result.error)
                else -> {
                    vmState.update { VoucherDialogUiState.Default }
                }
            }
        }
    }

    private fun handleAddedTime(timeAdded: Long) {
        var days = (timeAdded / SECONDS_PER_DAY).toInt()
        viewModelScope.launch {
            vmState.update {
                VoucherDialogUiState.Success(
                    if (days > 60) {
                        resources.getQuantityString(
                            R.plurals.months_added_to_your_account,
                            days / 60
                        )
                    } else {
                        resources.getQuantityString(R.plurals.days_added_to_your_account, days)
                    }
                )
            }
        }
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
            vmState.update { VoucherDialogUiState.Error(message) }
        }
    }
}

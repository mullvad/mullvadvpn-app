package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.state.VoucherDialogViewModelState
import net.mullvad.mullvadvpn.model.VoucherSubmissionError
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.VoucherRedeemer

class VoucherDialogViewModel(
    private val accountRepository: AccountRepository,
    private val serviceConnectionManager: ServiceConnectionManager,
    private val resources: Resources
) : ViewModel() {
    private val vmState =
        MutableStateFlow<VoucherDialogViewModelState>(VoucherDialogViewModelState.Default)
    private val _viewActions = MutableSharedFlow<Unit>()
    val viewActions = _viewActions.asSharedFlow()

    private var voucherRedeemer: VoucherRedeemer? = null

    var uiState =
        vmState
            .map { it.toUiState() }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                VoucherDialogViewModelState.Default.toUiState()
            )

    fun onRedeem(voucherCode: String) {
        vmState.update { VoucherDialogViewModelState.Verifying }
        serviceConnectionManager.serviceNotifier.subscribe(this) { connection ->
            voucherRedeemer = connection?.voucherRedeemer
            viewModelScope.launch {
                val result = voucherRedeemer?.submit(voucherCode)
                serviceConnectionManager.serviceNotifier.unsubscribe(this)
                when (result) {
                    is VoucherSubmissionResult.Ok -> handleAddedTime(result.submission.timeAdded)
                    is VoucherSubmissionResult.Error -> setError(result.error)
                    else -> {
                        vmState.update { VoucherDialogViewModelState.Default }
                    }
                }
            }
        }
    }

    private fun handleAddedTime(timeAdded: Long) {
        viewModelScope.launch {
            vmState.update {
                VoucherDialogViewModelState.Success(
                    resources.getString(R.string.days_added_to_your_account, timeAdded.toString())
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
            vmState.update { VoucherDialogViewModelState.Error(message) }
        }
    }
}

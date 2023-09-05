package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.VoucherDialogUiState
import net.mullvad.mullvadvpn.model.VoucherSubmissionError
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.VoucherRedeemer

class VoucherDialogViewModel(
    private val accountRepository: AccountRepository,
    private val serviceConnectionManager: ServiceConnectionManager,
) : ViewModel() {
    private val vmState = MutableStateFlow<String?>(null)
    private val _viewActions = MutableSharedFlow<Unit>()
    val viewActions = _viewActions.asSharedFlow()

    private var voucherRedeemer: VoucherRedeemer? = null

    var uiState =
        vmState
            .map { VoucherDialogUiState(message = it) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                VoucherDialogUiState(message = null)
            )

    fun onRedeem(voucherCode: String) {
        serviceConnectionManager.serviceNotifier.subscribe(this) { connection ->
            voucherRedeemer = connection?.voucherRedeemer
            viewModelScope.launch {
                val result = voucherRedeemer?.submit(voucherCode)
                serviceConnectionManager.serviceNotifier.unsubscribe(this)
                when (result) {
                    is VoucherSubmissionResult.Ok -> handleAddedTime(result.submission.timeAdded)
                    is VoucherSubmissionResult.Error -> setError(result.error)
                    else -> {
                        /* NOOP */
                    }
                }
            }
        }
    }

    private fun handleAddedTime(timeAdded: Long) {
        viewModelScope.launch { _viewActions.emit(Unit) }
    }

    private fun setError(error: VoucherSubmissionError) {
        //        viewModelScope.launch { _viewActions.emit(Unit) }
        //        val message =
        //            when (error) {
        //                VoucherSubmissionError.InvalidVoucher -> R.string.invalid_voucher
        //                VoucherSubmissionError.VoucherAlreadyUsed -> R.string.voucher_already_used
        //                else -> R.string.error_occurred
        //            }
        //
        //        errorMessage.apply {
        //            setText(message)
        //            visibility = View.VISIBLE
        //        }
    }
}

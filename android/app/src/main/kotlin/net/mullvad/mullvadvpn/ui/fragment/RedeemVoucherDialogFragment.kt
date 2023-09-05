package net.mullvad.mullvadvpn.ui.fragment

import android.app.Dialog
import android.content.Context
import android.graphics.drawable.ColorDrawable
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.view.ViewGroup.LayoutParams
import android.widget.TextView
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import androidx.fragment.app.DialogFragment
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.RedeemVoucherDialogScreen
import net.mullvad.mullvadvpn.lib.common.util.JobTracker
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.model.VoucherSubmissionError
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.VoucherRedeemer
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.viewmodel.VoucherDialogViewModel
import org.koin.android.ext.android.inject
import org.koin.androidx.viewmodel.ext.android.viewModel

class RedeemVoucherDialogFragment : DialogFragment() {

    // Injected dependencies
    private val accountRepository: AccountRepository by inject()
    private val serviceConnectionManager: ServiceConnectionManager by inject()

    private val jobTracker = JobTracker()

    private lateinit var errorMessage: TextView

    private var redeemButton: Button? = null
    private var voucherRedeemer: VoucherRedeemer? = null

    private val vm by viewModel<VoucherDialogViewModel>()
    private lateinit var voucherDialog: Dialog

    private var voucherInputIsValid = false
        set(value) {
            field = value
            updateRedeemButton()
        }

    override fun onAttach(context: Context) {
        super.onAttach(context)

        serviceConnectionManager.serviceNotifier.subscribe(this) { connection ->
            voucherRedeemer = connection?.voucherRedeemer
        }

        updateRedeemButton()
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        //        val view = inflater.inflate(R.layout.redeem_voucher, container, false)
        //
        //        redeemButton =
        //            view.findViewById<Button>(R.id.redeem).apply {
        //                setEnabled(false)
        //
        //                setOnClickAction("action", jobTracker) { submitVoucher() }
        //            }
        //
        //        errorMessage = view.findViewById(R.id.error)
        //
        //        view.findViewById<Button>(R.id.cancel).setOnClickAction("action", jobTracker) {
        //            activity?.onBackPressed()
        //        }
        //
        //        return view

        return inflater.inflate(R.layout.fragment_compose, container, false).apply {
            findViewById<ComposeView>(R.id.compose_view).setContent {
                AppTheme {
                    val state = vm.uiState.collectAsState().value
                    RedeemVoucherDialogScreen(
                        uiState = state,
                        onRedeem = { vm.onRedeem(it) },
                        onDismiss = { onDismiss(voucherDialog!!) }
                    )
                }
            }
        }
    }

    override fun onCreateDialog(savedInstanceState: Bundle?): Dialog {
        voucherDialog = super.onCreateDialog(savedInstanceState)
        voucherDialog.window?.setBackgroundDrawable(ColorDrawable(android.R.color.transparent))
        return voucherDialog
    }

    override fun onStart() {
        super.onStart()
        dialog?.window?.setLayout(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
    }

    override fun onDestroyView() {
        jobTracker.cancelAllJobs()
        super.onDestroyView()
    }

    override fun onDetach() {
        jobTracker.cancelJob("updateExpiry")
        serviceConnectionManager.serviceNotifier.unsubscribe(this)

        super.onDetach()
    }

    private fun updateRedeemButton() {
        redeemButton?.setEnabled(voucherInputIsValid && voucherRedeemer != null)
    }

    //    private suspend fun submitVoucher() {
    //        errorMessage.visibility = View.INVISIBLE
    //
    //        val result = voucherRedeemer?.submit(voucherInput.text.toString())
    //
    //        when (result) {
    //            is VoucherSubmissionResult.Ok -> handleAddedTime(result.submission.timeAdded)
    //            is VoucherSubmissionResult.Error -> showError(result.error)
    //            else -> {
    //                /* NOOP */
    //            }
    //        }
    //    }

    private fun handleAddedTime(timeAdded: Long) {
        if (timeAdded > 0) {
            dismiss()
        }
    }

    private fun showError(error: VoucherSubmissionError) {
        val message =
            when (error) {
                VoucherSubmissionError.InvalidVoucher -> R.string.invalid_voucher
                VoucherSubmissionError.VoucherAlreadyUsed -> R.string.voucher_already_used
                else -> R.string.error_occurred
            }

        errorMessage.apply {
            setText(message)
            visibility = View.VISIBLE
        }
    }
}

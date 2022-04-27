package net.mullvad.mullvadvpn.ui

import android.app.Dialog
import android.content.Context
import android.graphics.drawable.ColorDrawable
import android.os.Bundle
import android.text.Editable
import android.text.TextWatcher
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.view.ViewGroup.LayoutParams
import android.widget.EditText
import android.widget.TextView
import androidx.fragment.app.DialogFragment
import kotlinx.coroutines.flow.collect
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.VoucherSubmissionError
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountCache
import net.mullvad.mullvadvpn.ui.serviceconnection.VoucherRedeemer
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.mullvadvpn.util.SegmentedInputFormatter
import org.joda.time.DateTime

const val FULL_VOUCHER_CODE_LENGTH = "XXXX-XXXX-XXXX-XXXX".length

class RedeemVoucherDialogFragment : DialogFragment() {
    private val jobTracker = JobTracker()

    private lateinit var parentActivity: MainActivity
    private lateinit var errorMessage: TextView
    private lateinit var voucherInput: EditText

    private var accountCache: AccountCache? = null
    private var accountExpiry: DateTime? = null
    private var redeemButton: Button? = null
    private var voucherRedeemer: VoucherRedeemer? = null

    private var voucherInputIsValid = false
        set(value) {
            field = value
            updateRedeemButton()
        }

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity

        parentActivity.serviceNotifier.subscribe(this) { connection ->
            accountCache = connection?.accountCache
            voucherRedeemer = connection?.voucherRedeemer
        }

        accountCache?.apply {
            jobTracker.newUiJob("updateExpiry") {
                accountCache?.accountExpiryState?.collect { state ->
                    accountExpiry = state.date()
                }
            }
        }

        updateRedeemButton()
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.redeem_voucher, container, false)

        voucherInput = view.findViewById<EditText>(R.id.voucher_code).apply {
            addTextChangedListener(ValidVoucherCodeChecker())
        }

        SegmentedInputFormatter(voucherInput, '-').apply {
            allCaps = true

            isValidInputCharacter = { character ->
                ('A' <= character && character <= 'Z') || ('0' <= character && character <= '9')
            }
        }

        redeemButton = view.findViewById<Button>(R.id.redeem).apply {
            setEnabled(false)

            setOnClickAction("action", jobTracker) {
                submitVoucher()
            }
        }

        errorMessage = view.findViewById(R.id.error)

        view.findViewById<Button>(R.id.cancel).setOnClickAction("action", jobTracker) {
            activity?.onBackPressed()
        }

        return view
    }

    override fun onCreateDialog(savedInstanceState: Bundle?): Dialog {
        val dialog = super.onCreateDialog(savedInstanceState)

        dialog.window?.setBackgroundDrawable(ColorDrawable(android.R.color.transparent))

        return dialog
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
        parentActivity.serviceNotifier.unsubscribe(this)

        super.onDetach()
    }

    private fun updateRedeemButton() {
        redeemButton?.setEnabled(voucherInputIsValid && voucherRedeemer != null)
    }

    private suspend fun submitVoucher() {
        errorMessage.visibility = View.INVISIBLE

        val result = voucherRedeemer?.submit(voucherInput.text.toString())

        when (result) {
            is VoucherSubmissionResult.Ok -> handleAddedTime(result.submission.timeAdded)
            is VoucherSubmissionResult.Error -> showError(result.error)
        }
    }

    private fun handleAddedTime(timeAdded: Long) {
        if (timeAdded > 0) {
            accountExpiry?.let { oldAccountExpiry ->
                accountCache?.invalidateAccountExpiry(oldAccountExpiry)
            }

            dismiss()
        }
    }

    private fun showError(error: VoucherSubmissionError) {
        val message = when (error) {
            VoucherSubmissionError.InvalidVoucher -> R.string.invalid_voucher
            VoucherSubmissionError.VoucherAlreadyUsed -> R.string.voucher_already_used
            else -> R.string.error_occurred
        }

        errorMessage.apply {
            setText(message)
            visibility = View.VISIBLE
        }
    }

    inner class ValidVoucherCodeChecker : TextWatcher {
        private var editRecursionCount = 0

        override fun beforeTextChanged(text: CharSequence, start: Int, count: Int, after: Int) {
            editRecursionCount += 1
        }

        override fun onTextChanged(text: CharSequence, start: Int, before: Int, count: Int) {}

        override fun afterTextChanged(text: Editable) {
            editRecursionCount -= 1

            if (editRecursionCount == 0) {
                voucherInputIsValid = text.length == FULL_VOUCHER_CODE_LENGTH
            }
        }
    }
}

package net.mullvad.mullvadvpn.ui

import android.app.Dialog
import android.graphics.drawable.ColorDrawable
import android.os.Bundle
import android.support.v4.app.DialogFragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.view.ViewGroup.LayoutParams
import android.widget.EditText
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.mullvadvpn.util.SegmentedInputFormatter

class RedeemVoucherDialogFragment : DialogFragment() {
    private val jobTracker = JobTracker()

    private lateinit var voucherInput: EditText

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.redeem_voucher, container, false)

        voucherInput = view.findViewById(R.id.voucher_code)

        SegmentedInputFormatter(voucherInput, '-').apply {
            allCaps = true

            isValidInputCharacter = { character ->
                ('A' <= character && character <= 'Z') || ('0' <= character && character <= '9')
            }
        }

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
}

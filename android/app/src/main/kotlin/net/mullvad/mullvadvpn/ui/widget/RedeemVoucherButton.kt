package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import androidx.fragment.app.FragmentManager
import net.mullvad.mullvadvpn.ui.fragment.RedeemVoucherDialogFragment
import net.mullvad.mullvadvpn.util.JobTracker

class RedeemVoucherButton : Button {
    constructor(context: Context) : super(context)

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes)

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute)

    fun prepare(
        fragmentManager: FragmentManager?,
        jobTracker: JobTracker,
        jobName: String = "openRedeemVoucherDialog"
    ) {
        setOnClickAction(jobName, jobTracker) {
            fragmentManager?.beginTransaction()?.let { transaction ->
                transaction.addToBackStack(null)

                RedeemVoucherDialogFragment().show(transaction, null)
            }
        }
    }
}

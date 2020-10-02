package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.support.v4.app.FragmentManager
import android.util.AttributeSet
import net.mullvad.mullvadvpn.ui.RedeemVoucherDialogFragment
import net.mullvad.mullvadvpn.util.JobTracker

class RedeemVoucherButton : Button {
    constructor(context: Context) : super(context) {}

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes) {}

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute) {}

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {}

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

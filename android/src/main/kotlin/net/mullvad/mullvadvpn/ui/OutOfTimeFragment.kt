package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.ui.widget.UrlButton
import net.mullvad.mullvadvpn.util.JobTracker

class OutOfTimeFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    private val jobTracker = JobTracker()

    private lateinit var disconnectButton: Button

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.out_of_time, container, false)

        view.findViewById<View>(R.id.settings).setOnClickListener {
            parentActivity.openSettings()
        }

        disconnectButton = view.findViewById<Button>(R.id.disconnect).apply {
            setOnClickAction("disconnect", jobTracker) {
                connectionProxy.disconnect()
            }
        }

        view.findViewById<UrlButton>(R.id.buy_credit).apply {
            prepare(daemon, jobTracker)
        }

        view.findViewById<Button>(R.id.redeem_voucher).apply {
            setOnClickAction("openRedeemVoucherDialog", jobTracker) {
                showRedeemVoucherDialog()
            }
        }

        return view
    }

    override fun onSafelyDestroyView() {
        jobTracker.cancelAllJobs()
    }

    private fun showRedeemVoucherDialog() {
        val transaction = fragmentManager?.beginTransaction()

        transaction?.addToBackStack(null)

        RedeemVoucherDialogFragment().show(transaction, null)
    }
}

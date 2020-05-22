package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.ui.widget.UrlButton
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import org.joda.time.DateTime

class OutOfTimeFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    private lateinit var headerBar: HeaderBar

    private lateinit var buyCreditButton: UrlButton
    private lateinit var disconnectButton: Button
    private lateinit var redeemButton: Button

    private var tunnelState: TunnelState = TunnelState.Disconnected()
        set(value) {
            field = value
            updateDisconnectButton()
            updateBuyButtons()
            headerBar.setState(value)
        }

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.out_of_time, container, false)

        view.findViewById<View>(R.id.settings).setOnClickListener {
            parentActivity.openSettings()
        }

        headerBar = HeaderBar(view, resources)

        disconnectButton = view.findViewById<Button>(R.id.disconnect).apply {
            setOnClickAction("disconnect", jobTracker) {
                connectionProxy.disconnect()
            }
        }

        buyCreditButton = view.findViewById<UrlButton>(R.id.buy_credit).apply {
            prepare(daemon, jobTracker)
        }

        redeemButton = view.findViewById<Button>(R.id.redeem_voucher).apply {
            setOnClickAction("openRedeemVoucherDialog", jobTracker) {
                showRedeemVoucherDialog()
            }
        }

        connectionProxy.onStateChange.subscribe(this) { newState ->
            jobTracker.newUiJob("updateTunnelState") {
                tunnelState = newState
            }
        }

        return view
    }

    override fun onSafelyResume() {
        accountCache.onAccountExpiryChange.subscribe(this) { expiry ->
            checkExpiry(expiry)
        }

        jobTracker.newBackgroundJob("pollAccountData") {
            while (true) {
                accountCache.fetchAccountExpiry()
                delay(POLL_INTERVAL)
            }
        }
    }

    override fun onSafelyPause() {
        accountCache.onAccountExpiryChange.unsubscribe(this)
        jobTracker.cancelJob("pollAccountData")
    }

    override fun onSafelyDestroyView() {
        connectionProxy.onStateChange.unsubscribe(this)
    }

    private fun showRedeemVoucherDialog() {
        val transaction = fragmentManager?.beginTransaction()

        transaction?.addToBackStack(null)

        RedeemVoucherDialogFragment().show(transaction, null)
    }

    private fun updateDisconnectButton() {
        val state = tunnelState

        val showButton = when (state) {
            is TunnelState.Disconnected -> false
            is TunnelState.Connecting, is TunnelState.Connected -> true
            is TunnelState.Disconnecting -> {
                state.actionAfterDisconnect != ActionAfterDisconnect.Nothing
            }
            is TunnelState.Error -> state.errorState.isBlocking
        }

        disconnectButton.apply {
            if (showButton) {
                setEnabled(true)
                visibility = View.VISIBLE
            } else {
                setEnabled(false)
                visibility = View.GONE
            }
        }
    }

    private fun updateBuyButtons() {
        val hasConnectivity = tunnelState is TunnelState.Disconnected

        buyCreditButton.setEnabled(hasConnectivity)
        redeemButton.setEnabled(hasConnectivity)
    }

    private fun checkExpiry(maybeExpiry: DateTime?) {
        maybeExpiry?.let { expiry ->
            if (expiry.isAfterNow()) {
                jobTracker.newUiJob("advanceToConnectScreen") {
                    advanceToConnectScreen()
                }
            }
        }
    }

    private fun advanceToConnectScreen() {
        fragmentManager?.beginTransaction()?.apply {
            replace(R.id.main_fragment, ConnectFragment())
            commit()
        }
    }
}

package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.ui.widget.HeaderBar
import net.mullvad.mullvadvpn.ui.widget.RedeemVoucherButton
import net.mullvad.mullvadvpn.ui.widget.UrlButton
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import org.joda.time.DateTime

class OutOfTimeFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    private lateinit var headerBar: HeaderBar

    private lateinit var buyCreditButton: UrlButton
    private lateinit var disconnectButton: Button
    private lateinit var redeemButton: RedeemVoucherButton

    private var tunnelState by observable<TunnelState>(TunnelState.Disconnected()) { _, _, state ->
        updateDisconnectButton()
        updateBuyButtons()
        headerBar.tunnelState = state
    }

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.out_of_time, container, false)

        headerBar = view.findViewById<HeaderBar>(R.id.header_bar).apply {
            tunnelState = this@OutOfTimeFragment.tunnelState
        }

        view.findViewById<TextView>(R.id.account_credit_has_expired).text =
            parentActivity.getString(R.string.account_credit_has_expired) + " " +
            parentActivity.getString(R.string.add_time_to_account)

        disconnectButton = view.findViewById<Button>(R.id.disconnect).apply {
            setOnClickAction("disconnect", jobTracker) {
                connectionProxy.disconnect()
            }
        }

        buyCreditButton = view.findViewById<UrlButton>(R.id.buy_credit).apply {
            prepare(daemon, jobTracker)
        }

        redeemButton = view.findViewById<RedeemVoucherButton>(R.id.redeem_voucher).apply {
            prepare(fragmentManager, jobTracker)
        }

        connectionProxy.onStateChange.subscribe(this) { newState ->
            jobTracker.newUiJob("updateTunnelState") {
                tunnelState = newState
            }
        }

        return view
    }

    override fun onSafelyStart() {
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

    override fun onSafelyStop() {
        accountCache.onAccountExpiryChange.unsubscribe(this)
        jobTracker.cancelJob("pollAccountData")
    }

    override fun onSafelyDestroyView() {
        connectionProxy.onStateChange.unsubscribe(this)
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

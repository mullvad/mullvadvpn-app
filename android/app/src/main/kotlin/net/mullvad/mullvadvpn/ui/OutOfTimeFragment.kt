package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.ui.widget.HeaderBar
import net.mullvad.mullvadvpn.ui.widget.RedeemVoucherButton
import net.mullvad.mullvadvpn.ui.widget.SitePaymentButton
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.joda.time.DateTime

class OutOfTimeFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    private lateinit var headerBar: HeaderBar

    private lateinit var sitePaymentButton: SitePaymentButton
    private lateinit var disconnectButton: Button
    private lateinit var redeemButton: RedeemVoucherButton

    private var tunnelState by observable<TunnelState>(TunnelState.Disconnected) { _, _, state ->
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

        view.findViewById<TextView>(R.id.account_credit_has_expired).text = buildString {
            append(parentActivity.getString(R.string.account_credit_has_expired))
            append(" ")
            parentActivity.getString(R.string.add_time_to_account)
        }

        disconnectButton = view.findViewById<Button>(R.id.disconnect).apply {
            setOnClickAction("disconnect", jobTracker) {
                connectionProxy.disconnect()
            }
        }

        sitePaymentButton = view.findViewById<SitePaymentButton>(R.id.site_payment).apply {
            newAccount = false
            prepare(authTokenCache, jobTracker)
        }

        redeemButton = view.findViewById<RedeemVoucherButton>(R.id.redeem_voucher).apply {
            prepare(parentFragmentManager, jobTracker)
        }

        connectionProxy.onStateChange.subscribe(this) { newState ->
            jobTracker.newUiJob("updateTunnelState") {
                tunnelState = newState
            }
        }

        return view
    }

    override fun onSafelyStart() {
        jobTracker.newUiJob("updateAccountExpiry") {
            accountCache.accountExpiryState
                .map { state -> state.date() }
                .collect { expiryDate ->
                    checkExpiry(expiryDate)
                }
        }

        jobTracker.newBackgroundJob("pollAccountData") {
            while (true) {
                accountCache.fetchAccountExpiry()
                delay(POLL_INTERVAL)
            }
        }

        sitePaymentButton.updateAuthTokenCache(authTokenCache)
    }

    override fun onSafelyStop() {
        jobTracker.cancelJob("updateAccountExpiry")
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
        val currentState = tunnelState
        val hasConnectivity = currentState is TunnelState.Disconnected
        sitePaymentButton.setEnabled(hasConnectivity)

        val isOffline = currentState is TunnelState.Error &&
            currentState.errorState.cause is ErrorStateCause.IsOffline
        redeemButton.setEnabled(!isOffline)
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
        parentFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, ConnectFragment())
            commit()
        }
    }
}

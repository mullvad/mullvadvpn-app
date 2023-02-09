package net.mullvad.mullvadvpn.ui.fragment

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.ui.extension.openAccountPageInBrowser
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.ui.serviceconnection.connectionProxy
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.ui.widget.HeaderBar
import net.mullvad.mullvadvpn.ui.widget.RedeemVoucherButton
import net.mullvad.mullvadvpn.ui.widget.SitePaymentButton
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.joda.time.DateTime
import org.koin.android.ext.android.inject

class OutOfTimeFragment : BaseFragment() {

    // Injected dependencies
    private val accountRepository: AccountRepository by inject()
    private val serviceConnectionManager: ServiceConnectionManager by inject()

    private lateinit var headerBar: HeaderBar

    private lateinit var sitePaymentButton: SitePaymentButton
    private lateinit var disconnectButton: Button
    private lateinit var redeemButton: RedeemVoucherButton

    private var tunnelState by observable<TunnelState>(TunnelState.Disconnected) { _, _, state ->
        updateDisconnectButton()
        updateBuyButtons()
        headerBar.tunnelState = state
    }

    @Deprecated("Refactor code to instead rely on Lifecycle.")
    private val jobTracker = JobTracker()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        lifecycleScope.launchUiSubscriptionsOnResume()
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {
        val view = inflater.inflate(R.layout.out_of_time, container, false)

        headerBar = view.findViewById<HeaderBar>(R.id.header_bar).apply {
            tunnelState = this@OutOfTimeFragment.tunnelState
        }

        view.findViewById<TextView>(R.id.account_credit_has_expired).text = buildString {
            append(requireActivity().getString(R.string.account_credit_has_expired))
            append(" ")
            requireActivity().getString(R.string.add_time_to_account)
        }

        disconnectButton = view.findViewById<Button>(R.id.disconnect).apply {
            setOnClickAction("disconnect", jobTracker) {
                serviceConnectionManager.connectionProxy()?.disconnect()
            }
        }

        sitePaymentButton = view.findViewById<SitePaymentButton>(R.id.site_payment).apply {
            newAccount = false

            setOnClickAction("openAccountPageInBrowser", jobTracker) {
                setEnabled(false)
                serviceConnectionManager.authTokenCache()?.fetchAuthToken()?.let { token ->
                    context.openAccountPageInBrowser(token)
                }
                setEnabled(true)
            }

            isEnabled = true
        }

        redeemButton = view.findViewById<RedeemVoucherButton>(R.id.redeem_voucher).apply {
            prepare(parentFragmentManager, jobTracker)
        }

        return view
    }

    override fun onStop() {
        jobTracker.cancelAllJobs()
        super.onStop()
    }

    private fun CoroutineScope.launchUiSubscriptionsOnResume() = launch {
        repeatOnLifecycle(Lifecycle.State.RESUMED) {
            launchProceedToConnectViewIfExpiryExtended()
            launchExpiryPolling()
            launchTunnelStateSubscription()
        }
    }

    private fun CoroutineScope.launchProceedToConnectViewIfExpiryExtended() = launch {
        accountRepository.accountExpiryState
            .map { state -> state.date() }
            .collect { expiryDate ->
                checkExpiry(expiryDate)
            }
    }

    private fun CoroutineScope.launchExpiryPolling() = launch {
        while (true) {
            accountRepository.fetchAccountExpiry()
            delay(POLL_INTERVAL)
        }
    }

    private fun CoroutineScope.launchTunnelStateSubscription() = launch {
        serviceConnectionManager.connectionState
            .flatMapLatest { state ->
                if (state is ServiceConnectionState.ConnectedReady) {
                    callbackFlowFromNotifier(
                        state.container.connectionProxy.onStateChange
                    )
                } else {
                    emptyFlow()
                }
            }
            .collect { newState ->
                tunnelState = newState
            }
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
            commitAllowingStateLoss()
        }
    }
}

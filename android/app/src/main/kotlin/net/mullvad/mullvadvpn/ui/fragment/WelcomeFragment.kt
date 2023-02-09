package net.mullvad.mullvadvpn.ui.fragment

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import android.widget.Toast
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.extension.openAccountPageInBrowser
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.authTokenCache
import net.mullvad.mullvadvpn.ui.widget.HeaderBar
import net.mullvad.mullvadvpn.ui.widget.RedeemVoucherButton
import net.mullvad.mullvadvpn.ui.widget.SitePaymentButton
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.mullvadvpn.util.UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS
import net.mullvad.mullvadvpn.util.addDebounceForUnknownState
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import org.joda.time.DateTime
import org.koin.android.ext.android.inject

val POLL_INTERVAL: Long = 15 /* s */ * 1000 /* ms */

class WelcomeFragment : BaseFragment() {

    // Injected dependencies
    private val accountRepository: AccountRepository by inject()
    private val deviceRepository: DeviceRepository by inject()
    private val serviceConnectionManager: ServiceConnectionManager by inject()

    private lateinit var accountLabel: TextView
    private lateinit var headerBar: HeaderBar
    private lateinit var sitePaymentButton: SitePaymentButton

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
        val view = inflater.inflate(R.layout.welcome, container, false)

        headerBar = view.findViewById<HeaderBar>(R.id.header_bar).apply {
            tunnelState = TunnelState.Disconnected
        }

        accountLabel = view.findViewById<TextView>(R.id.account_number).apply {
            setOnClickListener { copyAccountTokenToClipboard() }
        }

        view.findViewById<TextView>(R.id.pay_to_start_using).text = buildString {
            append(requireActivity().getString(R.string.pay_to_start_using))
            append(" ")
            append(requireActivity().getString(R.string.add_time_to_account))
        }

        sitePaymentButton = view.findViewById<SitePaymentButton>(R.id.site_payment).apply {
            newAccount = true

            setOnClickAction("openAccountPageInBrowser", jobTracker) {
                setEnabled(false)
                serviceConnectionManager.authTokenCache()?.fetchAuthToken()?.let { token ->
                    context.openAccountPageInBrowser(token)
                }
                setEnabled(true)
            }
        }

        view.findViewById<RedeemVoucherButton>(R.id.redeem_voucher).apply {
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
            launchUpdateAccountNumberOnDeviceChanges()
            launchAdvanceToConnectViewOnExpiryExtended()
            launchExpiryPolling()
            launchTunnelStateSubscription()
        }
    }

    private fun CoroutineScope.launchUpdateAccountNumberOnDeviceChanges() = launch {
        deviceRepository.deviceState
            .debounce { it.addDebounceForUnknownState(UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS) }
            .collect { state ->
                updateAccountNumber(state.token())
            }
    }

    private fun CoroutineScope.launchAdvanceToConnectViewOnExpiryExtended() = launch {
        accountRepository.accountExpiryState.collect {
            checkExpiry(it.date())
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
            .collect { state -> updateUiForTunnelState(state) }
    }

    private fun updateUiForTunnelState(tunnelState: TunnelState) {
        headerBar.tunnelState = tunnelState
        sitePaymentButton.isEnabled = tunnelState is TunnelState.Disconnected
    }

    private fun updateAccountNumber(rawAccountNumber: String?) {
        val accountText = rawAccountNumber?.let { account ->
            addSpacesToAccountText(account)
        }

        accountLabel.text = accountText ?: ""
        accountLabel.setEnabled(accountText != null && accountText.length > 0)
    }

    private fun addSpacesToAccountText(account: String): String {
        val length = account.length

        if (length == 0) {
            return ""
        } else {
            val numParts = (length - 1) / 4 + 1

            val parts = Array(numParts) { index ->
                val startIndex = index * 4
                val endIndex = minOf(startIndex + 4, length)

                account.substring(startIndex, endIndex)
            }

            return parts.joinToString(" ")
        }
    }

    private fun checkExpiry(maybeExpiry: DateTime?) {
        maybeExpiry?.let { expiry ->
            val tomorrow = DateTime.now().plusDays(1)

            if (expiry.isAfter(tomorrow)) {
                advanceToConnectScreen()
            }
        }
    }

    private fun advanceToConnectScreen() {
        parentFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, ConnectFragment())
            commitAllowingStateLoss()
        }
    }

    private fun copyAccountTokenToClipboard() {
        val accountToken = accountLabel.text
        val clipboardLabel = resources.getString(R.string.mullvad_account_number)
        val toastMessage = resources.getString(R.string.copied_mullvad_account_number)

        val context = requireActivity()
        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        val clipData = ClipData.newPlainText(clipboardLabel, accountToken)

        clipboard.setPrimaryClip(clipData)

        Toast.makeText(context, toastMessage, Toast.LENGTH_SHORT).show()
    }
}

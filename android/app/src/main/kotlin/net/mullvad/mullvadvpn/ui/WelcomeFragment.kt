package net.mullvad.mullvadvpn.ui

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import android.widget.Toast
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collect
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.widget.HeaderBar
import net.mullvad.mullvadvpn.ui.widget.RedeemVoucherButton
import net.mullvad.mullvadvpn.ui.widget.SitePaymentButton
import org.joda.time.DateTime

val POLL_INTERVAL: Long = 15 /* s */ * 1000 /* ms */

class WelcomeFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    private lateinit var accountLabel: TextView
    private lateinit var sitePaymentButton: SitePaymentButton

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.welcome, container, false)

        view.findViewById<HeaderBar>(R.id.header_bar).apply {
            tunnelState = TunnelState.Disconnected
        }

        accountLabel = view.findViewById<TextView>(R.id.account_number).apply {
            setOnClickListener { copyAccountTokenToClipboard() }
        }

        view.findViewById<TextView>(R.id.pay_to_start_using).text = buildString {
            append(parentActivity.getString(R.string.pay_to_start_using))
            append(" ")
            append(parentActivity.getString(R.string.add_time_to_account))
        }

        sitePaymentButton = view.findViewById<SitePaymentButton>(R.id.site_payment).apply {
            newAccount = true
            prepare(authTokenCache, jobTracker)
        }

        view.findViewById<RedeemVoucherButton>(R.id.redeem_voucher).apply {
            prepare(parentFragmentManager, jobTracker)
        }

        return view
    }

    override fun onSafelyStart() {
        jobTracker.newUiJob("updateAccountNumber") {
            deviceRepository.deviceState.collect { state ->
                updateAccountNumber(state.token())
            }
        }

        jobTracker.newUiJob("checkAccountExpiry") {
            accountCache.accountExpiryState.collect {
                checkExpiry(it.date())
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
        jobTracker.cancelJob("checkAccountExpiry")
        jobTracker.cancelJob("pollAccountData")
        jobTracker.cancelJob("updateAccountNumber")
    }

    private fun updateAccountNumber(rawAccountNumber: String?) {
        val accountText = rawAccountNumber?.let { account ->
            addSpacesToAccountText(account)
        }

        jobTracker.newUiJob("updateAccountNumber") {
            accountLabel.text = accountText ?: ""
            accountLabel.setEnabled(accountText != null && accountText.length > 0)
        }
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

    private fun copyAccountTokenToClipboard() {
        val accountToken = accountLabel.text
        val clipboardLabel = resources.getString(R.string.mullvad_account_number)
        val toastMessage = resources.getString(R.string.copied_mullvad_account_number)

        val context = parentActivity
        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        val clipData = ClipData.newPlainText(clipboardLabel, accountToken)

        clipboard.setPrimaryClip(clipData)

        Toast.makeText(context, toastMessage, Toast.LENGTH_SHORT).show()
    }
}

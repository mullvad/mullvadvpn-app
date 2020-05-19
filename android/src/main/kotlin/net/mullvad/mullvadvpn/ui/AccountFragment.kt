package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.support.v4.app.FragmentManager
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import java.text.DateFormat
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.ui.widget.CopyableInformationView
import net.mullvad.mullvadvpn.ui.widget.InformationView
import net.mullvad.mullvadvpn.ui.widget.UrlButton
import org.joda.time.DateTime

class AccountFragment : ServiceDependentFragment(OnNoService.GoBack) {
    private val dateStyle = DateFormat.MEDIUM
    private val timeStyle = DateFormat.SHORT
    private val expiryFormatter = DateFormat.getDateTimeInstance(dateStyle, timeStyle)

    private lateinit var accountExpiryView: InformationView
    private lateinit var accountNumberView: CopyableInformationView

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.account, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            parentActivity.onBackPressed()
        }

        view.findViewById<UrlButton>(R.id.buy_credit).prepare(daemon, jobTracker)

        view.findViewById<Button>(R.id.redeem_voucher).setOnClickAction("redeem", jobTracker) {
            showRedeemVoucherDialog()
        }

        view.findViewById<Button>(R.id.logout).setOnClickAction("logout", jobTracker) {
            logout()
        }

        accountNumberView = view.findViewById<CopyableInformationView>(R.id.account_number).apply {
            displayFormatter = { rawAccountNumber -> addSpacesToAccountNumber(rawAccountNumber) }
        }

        accountExpiryView = view.findViewById(R.id.account_expiry)

        return view
    }

    override fun onSafelyResume() {
        accountCache.onAccountNumberChange.subscribe(this) { accountNumber ->
            jobTracker.newUiJob("updateAccountNumber") {
                accountNumberView.information = accountNumber
            }
        }

        accountCache.onAccountExpiryChange.subscribe(this) { accountExpiry ->
            jobTracker.newUiJob("updateAccountExpiry") {
                updateAccountExpiry(accountExpiry)
            }
        }
    }

    override fun onSafelyPause() {
        accountCache.onAccountNumberChange.unsubscribe(this)
        accountCache.onAccountExpiryChange.unsubscribe(this)
    }

    private fun updateAccountExpiry(accountExpiry: DateTime?) {
        if (accountExpiry != null) {
            accountExpiryView.information = expiryFormatter.format(accountExpiry.toDate())
        } else {
            accountExpiryView.information = null
            accountCache.fetchAccountExpiry()
        }
    }

    private fun showRedeemVoucherDialog() {
        val transaction = fragmentManager?.beginTransaction()

        transaction?.addToBackStack(null)

        RedeemVoucherDialogFragment().show(transaction, null)
    }

    private suspend fun logout() {
        clearAccountNumber()
        clearBackStack()
        goToLoginScreen()
    }

    private suspend fun clearAccountNumber() {
        jobTracker.runOnBackground {
            daemon.setAccount(null)
        }
    }

    private fun clearBackStack() {
        fragmentManager?.apply {
            val firstEntry = getBackStackEntryAt(0)

            popBackStack(firstEntry.id, FragmentManager.POP_BACK_STACK_INCLUSIVE)
        }
    }

    private fun goToLoginScreen() {
        fragmentManager?.beginTransaction()?.apply {
            setCustomAnimations(
                R.anim.do_nothing,
                R.anim.fragment_exit_to_bottom,
                R.anim.do_nothing,
                R.anim.do_nothing
            )
            replace(R.id.main_fragment, LoginFragment())
            commit()
        }
    }

    private fun addSpacesToAccountNumber(rawAccountNumber: String): String {
        return rawAccountNumber
            .asSequence()
            .fold(StringBuilder()) { formattedAccountNumber, nextDigit ->
                if ((formattedAccountNumber.length % 5) == 4) {
                    formattedAccountNumber.append(' ')
                }

                formattedAccountNumber.append(nextDigit)
            }
            .toString()
    }
}

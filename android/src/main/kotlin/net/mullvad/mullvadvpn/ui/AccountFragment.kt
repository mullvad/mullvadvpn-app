package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.support.v4.app.FragmentManager
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import java.text.DateFormat
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.widget.CopyableInformationView
import net.mullvad.mullvadvpn.ui.widget.InformationView
import org.joda.time.DateTime

class AccountFragment : ServiceDependentFragment(OnNoService.GoBack) {
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

        view.findViewById<View>(R.id.logout).setOnClickListener { logout() }

        accountNumberView = view.findViewById<CopyableInformationView>(R.id.account_number).apply {
            displayFormatter = { rawAccountNumber -> addSpacesToAccountNumber(rawAccountNumber) }
        }

        accountExpiryView = view.findViewById(R.id.account_expiry)

        return view
    }

    override fun onSafelyResume() {
        accountCache.onAccountDataChange = { accountNumber, accountExpiry ->
            jobTracker.newUiJob("updateView") {
                updateView(accountNumber, accountExpiry)
            }
        }
    }

    override fun onSafelyPause() {
        accountCache.onAccountDataChange = null
    }

    private fun updateView(accountNumber: String?, accountExpiry: DateTime?) {
        accountNumberView.information = accountNumber

        if (accountExpiry != null) {
            accountExpiryView.information = formatExpiry(accountExpiry)
        } else {
            accountExpiryView.information = null
            accountCache.fetchAccountExpiry()
        }
    }

    private fun formatExpiry(expiry: DateTime): String {
        val expiryInstant = expiry.toDate()
        val formatter = DateFormat.getDateTimeInstance()

        return formatter.format(expiryInstant)
    }

    private fun logout() {
        clearAccountNumber()
        clearBackStack()
        goToLoginScreen()
    }

    private fun clearAccountNumber() {
        jobTracker.newBackgroundJob("clearAccountNumber") {
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

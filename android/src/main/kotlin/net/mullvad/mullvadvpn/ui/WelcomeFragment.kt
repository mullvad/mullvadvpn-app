package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.JobTracker

class WelcomeFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    private val jobTracker = JobTracker()

    private lateinit var accountLabel: TextView

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.welcome, container, false)

        view.findViewById<View>(R.id.settings).setOnClickListener {
            parentActivity.openSettings()
        }

        accountLabel = view.findViewById<TextView>(R.id.account_number)

        return view
    }

    override fun onSafelyResume() {
        accountCache.onAccountDataChange = { account, _ -> updateAccountNumber(account) }
    }

    override fun onSafelyPause() {
        accountCache.onAccountDataChange = null
    }

    override fun onSafelyDestroyView() {
        jobTracker.cancelAllJobs()
    }

    private fun updateAccountNumber(rawAccountNumber: String?) {
        val accountText = rawAccountNumber?.let { account ->
            addSpacesToAccountText(account)
        }

        jobTracker.newUiJob("updateAccountNumber") {
            accountLabel.text = accountText ?: ""
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
}

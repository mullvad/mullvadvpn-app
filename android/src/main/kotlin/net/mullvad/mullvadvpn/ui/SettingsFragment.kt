package net.mullvad.mullvadvpn.ui

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.ImageButton
import android.widget.TextView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.service.AccountCache

class SettingsFragment : ServiceAwareFragment() {
    private lateinit var accountMenu: View
    private lateinit var appVersionWarning: View
    private lateinit var appVersionLabel: TextView
    private lateinit var appVersionFooter: View
    private lateinit var preferencesMenu: View
    private lateinit var advancedMenu: View
    private lateinit var remainingTimeLabel: RemainingTimeLabel

    private var active = false

    private var accountCache: AccountCache? = null
    private var versionInfoCache: AppVersionInfoCache? = null

    override fun onNewServiceConnection(serviceConnection: ServiceConnection) {
        accountCache = serviceConnection.accountCache
        versionInfoCache = serviceConnection.appVersionInfoCache

        if (active) {
            configureListeners()
        }
    }

    override fun onNoServiceConnection() {
        accountCache = null
        versionInfoCache = null
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.settings, container, false)

        view.findViewById<ImageButton>(R.id.close).setOnClickListener {
            activity?.onBackPressed()
        }

        view.findViewById<Button>(R.id.quit_button).setOnClickListener {
            parentActivity.quit()
        }

        accountMenu = view.findViewById<View>(R.id.account).apply {
            setOnClickListener {
                openSubFragment(AccountFragment())
            }
        }

        preferencesMenu = view.findViewById<View>(R.id.preferences).apply {
            setOnClickListener {
                openSubFragment(PreferencesFragment())
            }
        }

        advancedMenu = view.findViewById<View>(R.id.advanced).apply {
            setOnClickListener {
                openSubFragment(AdvancedFragment())
            }
        }

        view.findViewById<View>(R.id.app_version).setOnClickListener {
            openLink(R.string.download_url)
        }

        view.findViewById<View>(R.id.report_a_problem).setOnClickListener {
            openSubFragment(ProblemReportFragment())
        }

        appVersionWarning = view.findViewById(R.id.app_version_warning)
        appVersionLabel = view.findViewById<TextView>(R.id.app_version_label)
        appVersionFooter = view.findViewById(R.id.app_version_footer)
        remainingTimeLabel = RemainingTimeLabel(parentActivity, view)

        return view
    }

    override fun onResume() {
        super.onResume()

        configureListeners()
        active = true
    }

    override fun onPause() {
        active = false
        versionInfoCache?.onUpdate = null

        accountCache?.apply {
            onAccountNumberChange.unsubscribe(this@SettingsFragment)
            onAccountExpiryChange.unsubscribe(this@SettingsFragment)
        }

        super.onPause()
    }

    private fun configureListeners() {
        accountCache?.apply {
            onAccountNumberChange.subscribe(this@SettingsFragment) { account ->
                jobTracker.newUiJob("updateLoggedInStatus") {
                    updateLoggedInStatus(account != null)
                }
            }

            onAccountExpiryChange.subscribe(this@SettingsFragment) { expiry ->
                jobTracker.newUiJob("updateAccountInfo") {
                    remainingTimeLabel.accountExpiry = expiry
                }
            }

            fetchAccountExpiry()
        }

        versionInfoCache?.onUpdate = {
            jobTracker.newUiJob("updateVersionInfo") {
                updateVersionInfo()
            }
        }
    }

    private fun openSubFragment(fragment: Fragment) {
        fragmentManager?.beginTransaction()?.apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_right,
                R.anim.fragment_half_exit_to_left,
                R.anim.fragment_half_enter_from_left,
                R.anim.fragment_exit_to_right
            )
            replace(R.id.main_fragment, fragment)
            addToBackStack(null)
            commit()
        }
    }

    private fun openLink(urlResourceId: Int) {
        val intent = Intent(Intent.ACTION_VIEW, Uri.parse(parentActivity.getString(urlResourceId)))

        startActivity(intent)
    }

    private fun updateLoggedInStatus(loggedIn: Boolean) {
        val visibility = if (loggedIn) {
            View.VISIBLE
        } else {
            View.GONE
        }

        accountMenu.visibility = visibility
        preferencesMenu.visibility = visibility
        advancedMenu.visibility = visibility
    }

    private fun updateVersionInfo() {
        val isOutdated = versionInfoCache?.isOutdated ?: false
        val isSupported = versionInfoCache?.isSupported ?: true

        appVersionLabel.setText(versionInfoCache?.version ?: "")

        if (!isOutdated && isSupported) {
            appVersionWarning.visibility = View.GONE
            appVersionFooter.visibility = View.GONE
        } else {
            appVersionWarning.visibility = View.VISIBLE
            appVersionFooter.visibility = View.VISIBLE
        }
    }
}

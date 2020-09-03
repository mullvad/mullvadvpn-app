package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.ImageButton
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.service.AccountCache
import net.mullvad.mullvadvpn.ui.widget.AccountCell
import net.mullvad.mullvadvpn.ui.widget.AppVersionCell
import net.mullvad.mullvadvpn.ui.widget.NavigateCell

class SettingsFragment : ServiceAwareFragment() {
    private lateinit var accountMenu: AccountCell
    private lateinit var appVersionMenu: AppVersionCell
    private lateinit var preferencesMenu: View
    private lateinit var advancedMenu: View
    private lateinit var titleController: CollapsibleTitleController

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

        accountMenu = view.findViewById<AccountCell>(R.id.account).apply {
            targetFragment = AccountFragment::class
        }

        preferencesMenu = view.findViewById<NavigateCell>(R.id.preferences).apply {
            targetFragment = PreferencesFragment::class
        }

        advancedMenu = view.findViewById<NavigateCell>(R.id.advanced).apply {
            targetFragment = AdvancedFragment::class
        }

        view.findViewById<NavigateCell>(R.id.report_a_problem).apply {
            targetFragment = ProblemReportFragment::class
        }

        appVersionMenu = view.findViewById<AppVersionCell>(R.id.app_version)

        titleController = CollapsibleTitleController(view)

        return view
    }

    override fun onStart() {
        super.onStart()

        configureListeners()
        active = true
    }

    override fun onStop() {
        active = false
        versionInfoCache?.onUpdate = null

        accountCache?.apply {
            onAccountNumberChange.unsubscribe(this@SettingsFragment)
            onAccountExpiryChange.unsubscribe(this@SettingsFragment)
        }

        super.onStop()
    }

    override fun onDestroyView() {
        super.onDestroyView()
        titleController.onDestroy()
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
                    accountMenu.accountExpiry = expiry
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

        appVersionMenu.updateAvailable = isOutdated || !isSupported
        appVersionMenu.version = versionInfoCache?.version ?: ""
    }
}

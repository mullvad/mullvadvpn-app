package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ImageButton
import androidx.core.content.ContextCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.widget.AccountCell
import net.mullvad.mullvadvpn.ui.widget.AppVersionCell
import net.mullvad.mullvadvpn.ui.widget.NavigateCell
import org.koin.android.ext.android.inject

class SettingsFragment : ServiceAwareFragment(), StatusBarPainter, NavigationBarPainter {
    private val accountRepository: AccountRepository by inject()
    private val deviceRepository: DeviceRepository by inject()

    private lateinit var accountMenu: AccountCell
    private lateinit var appVersionMenu: AppVersionCell
    private lateinit var preferencesMenu: View
    private lateinit var advancedMenu: View
    private lateinit var titleController: CollapsibleTitleController

    private var active = false

    private var versionInfoCache: AppVersionInfoCache? = null

    override fun onNewServiceConnection(serviceConnectionContainer: ServiceConnectionContainer) {
        versionInfoCache = serviceConnectionContainer.appVersionInfoCache

        if (active) {
            configureListeners()
        }
    }

    override fun onNoServiceConnection() {
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

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        lifecycleScope.launchUiSubscriptionsOnResume()
    }

    override fun onResume() {
        super.onResume()
        paintNavigationBar(ContextCompat.getColor(requireContext(), R.color.darkBlue))
    }

    override fun onStart() {
        super.onStart()

        configureListeners()
        active = true
    }

    override fun onStop() {
        active = false
        versionInfoCache?.onUpdate = null

        jobTracker.cancelAllJobs()

        super.onStop()
    }

    override fun onDestroyView() {
        super.onDestroyView()
        titleController.onDestroy()
    }

    private fun CoroutineScope.launchUiSubscriptionsOnResume() = launch {
        repeatOnLifecycle(Lifecycle.State.RESUMED) {
            launchPaintStatusBarAfterTransition()
            luanchConfigureMenuOnDeviceChanges()
            launchUpdateExpiryTextOnExpiryChanges()
        }
    }

    private fun CoroutineScope.launchPaintStatusBarAfterTransition() = launch {
        transitionFinishedFlow.collect {
            paintStatusBar(ContextCompat.getColor(requireContext(), R.color.darkBlue))
        }
    }

    private fun CoroutineScope.luanchConfigureMenuOnDeviceChanges() = launch {
        deviceRepository.deviceState
            .collect { device ->
                updateLoggedInStatus(device is DeviceState.LoggedIn)
            }
    }

    private fun CoroutineScope.launchUpdateExpiryTextOnExpiryChanges() = launch {
        accountRepository.accountExpiryState
            .onStart { accountRepository.fetchAccountExpiry() }
            .map { state -> state.date() }
            .collect { expiryDate ->
                accountMenu.accountExpiry = expiryDate
            }
    }

    private fun configureListeners() {
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

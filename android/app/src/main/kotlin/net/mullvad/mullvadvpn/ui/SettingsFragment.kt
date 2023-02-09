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
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.ui.fragment.AccountFragment
import net.mullvad.mullvadvpn.ui.fragment.BaseFragment
import net.mullvad.mullvadvpn.ui.fragment.ProblemReportFragment
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.appVersionInfoCache
import net.mullvad.mullvadvpn.ui.widget.AccountCell
import net.mullvad.mullvadvpn.ui.widget.AppVersionCell
import net.mullvad.mullvadvpn.ui.widget.NavigateCell
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.mullvadvpn.util.UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS
import net.mullvad.mullvadvpn.util.addDebounceForUnknownState
import net.mullvad.mullvadvpn.util.appVersionCallbackFlow
import org.koin.android.ext.android.inject

class SettingsFragment : BaseFragment(), StatusBarPainter, NavigationBarPainter {

    // Injected dependencies
    private val accountRepository: AccountRepository by inject()
    private val deviceRepository: DeviceRepository by inject()
    private val serviceConnectionManager: ServiceConnectionManager by inject()

    private lateinit var accountMenu: AccountCell
    private lateinit var appVersionMenu: AppVersionCell
    private lateinit var preferencesMenu: View
    private lateinit var advancedMenu: View
    private lateinit var titleController: CollapsibleTitleController

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
        initializeUiState()
    }

    override fun onResume() {
        super.onResume()
        paintNavigationBar(ContextCompat.getColor(requireContext(), R.color.darkBlue))
    }

    override fun onStop() {
        jobTracker.cancelAllJobs()
        super.onStop()
    }

    override fun onDestroyView() {
        titleController.onDestroy()
        super.onDestroyView()
    }

    private fun initializeUiState() {
        updateLoggedInStatus(deviceRepository.deviceState.value is DeviceState.LoggedIn)
        accountMenu.accountExpiry = accountRepository.accountExpiryState.value.date()
        serviceConnectionManager.appVersionInfoCache().let { cache ->
            updateVersionInfo(
                if (cache != null) {
                    VersionInfo(
                        currentVersion = cache.version,
                        upgradeVersion = cache.upgradeVersion,
                        isOutdated = cache.isOutdated,
                        isSupported = cache.isSupported
                    )
                } else {
                    VersionInfo(
                        currentVersion = null,
                        upgradeVersion = null,
                        isOutdated = false,
                        isSupported = true
                    )
                }
            )
        }
    }

    private fun CoroutineScope.launchUiSubscriptionsOnResume() = launch {
        repeatOnLifecycle(Lifecycle.State.RESUMED) {
            launchPaintStatusBarAfterTransition()
            luanchConfigureMenuOnDeviceChanges()
            launchUpdateExpiryTextOnExpiryChanges()
            launchVersionInfoSubscription()
        }
    }

    private fun CoroutineScope.launchPaintStatusBarAfterTransition() = launch {
        transitionFinishedFlow.collect {
            paintStatusBar(ContextCompat.getColor(requireContext(), R.color.darkBlue))
        }
    }

    private fun CoroutineScope.luanchConfigureMenuOnDeviceChanges() = launch {
        deviceRepository.deviceState
            .debounce {
                it.addDebounceForUnknownState(UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS)
            }
            .collect { device ->
                updateLoggedInStatus(device is DeviceState.LoggedIn)
            }
    }

    private fun CoroutineScope.launchUpdateExpiryTextOnExpiryChanges() = launch {
        accountRepository.accountExpiryState
            .map { state -> state.date() }
            .collect { expiryDate ->
                accountMenu.accountExpiry = expiryDate
            }
    }

    private fun CoroutineScope.launchVersionInfoSubscription() = launch {
        serviceConnectionManager.connectionState
            .flatMapLatest { state ->
                if (state is ServiceConnectionState.ConnectedReady) {
                    state.container.appVersionInfoCache.appVersionCallbackFlow()
                } else {
                    emptyFlow()
                }
            }
            .collect { versionInfo ->
                updateVersionInfo(versionInfo)
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

    private fun updateVersionInfo(
        versionInfo: VersionInfo
    ) {
        appVersionMenu.updateAvailable = versionInfo.isOutdated || !versionInfo.isSupported
        appVersionMenu.version = versionInfo.currentVersion ?: ""
    }
}

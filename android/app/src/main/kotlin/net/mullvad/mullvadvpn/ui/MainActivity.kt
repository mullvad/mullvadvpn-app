package net.mullvad.mullvadvpn.ui

import android.app.Activity
import android.app.UiModeManager
import android.content.Intent
import android.content.pm.ActivityInfo
import android.content.res.Configuration
import android.net.VpnService
import android.os.Bundle
import android.util.Log
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.platform.ViewCompositionStrategy
import androidx.fragment.app.FragmentActivity
import androidx.fragment.app.FragmentManager
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onSubscription
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.dialog.ChangelogDialog
import net.mullvad.mullvadvpn.di.uiModule
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils.getNotificationPermissionResource
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils.isNotificationPermissionGranted
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.lib.endpoint.getApiEndpointConfigurationExtras
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository
import net.mullvad.mullvadvpn.ui.fragment.AccountFragment
import net.mullvad.mullvadvpn.ui.fragment.ConnectFragment
import net.mullvad.mullvadvpn.ui.fragment.DeviceRevokedFragment
import net.mullvad.mullvadvpn.ui.fragment.LoadingFragment
import net.mullvad.mullvadvpn.ui.fragment.LoginFragment
import net.mullvad.mullvadvpn.ui.fragment.OutOfTimeFragment
import net.mullvad.mullvadvpn.ui.fragment.PrivacyDisclaimerFragment
import net.mullvad.mullvadvpn.ui.fragment.SettingsFragment
import net.mullvad.mullvadvpn.ui.fragment.WelcomeFragment
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.util.UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS
import net.mullvad.mullvadvpn.util.addDebounceForUnknownState
import net.mullvad.mullvadvpn.viewmodel.ChangelogDialogUiState
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import org.koin.android.ext.android.getKoin
import org.koin.core.context.loadKoinModules

open class MainActivity : FragmentActivity() {
    private val requestNotificationPermissionLauncher =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) {
            // NotificationManager.areNotificationsEnabled is used to check the state rather than
            // handling the callback value.
        }

    private val deviceIsTv by lazy {
        val uiModeManager = getSystemService(UI_MODE_SERVICE) as UiModeManager

        uiModeManager.currentModeType == Configuration.UI_MODE_TYPE_TELEVISION
    }

    private lateinit var accountRepository: AccountRepository
    private lateinit var deviceRepository: DeviceRepository
    private lateinit var privacyDisclaimerRepository: PrivacyDisclaimerRepository
    private lateinit var serviceConnectionManager: ServiceConnectionManager
    private lateinit var changelogViewModel: ChangelogViewModel

    private var deviceStateJob: Job? = null
    private var currentDeviceState: DeviceState? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        loadKoinModules(uiModule)

        getKoin().apply {
            accountRepository = get()
            deviceRepository = get()
            privacyDisclaimerRepository = get()
            serviceConnectionManager = get()
            changelogViewModel = get()
        }

        requestedOrientation =
            if (deviceIsTv) {
                ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
            } else {
                ActivityInfo.SCREEN_ORIENTATION_PORTRAIT
            }

        super.onCreate(savedInstanceState)

        setContentView(R.layout.main)
    }

    override fun onStart() {
        Log.d("mullvad", "Starting main activity")
        super.onStart()

        if (privacyDisclaimerRepository.hasAcceptedPrivacyDisclosure()) {
            initializeStateHandlerAndServiceConnection(
                apiEndpointConfiguration = intent?.getApiEndpointConfigurationExtras()
            )
        } else {
            openPrivacyDisclaimerFragment()
        }
    }

    fun initializeStateHandlerAndServiceConnection(
        apiEndpointConfiguration: ApiEndpointConfiguration?
    ) {
        deviceStateJob = launchDeviceStateHandler()
        checkForNotificationPermission()
        serviceConnectionManager.bind(
            vpnPermissionRequestHandler = ::requestVpnPermission,
            apiEndpointConfiguration = apiEndpointConfiguration
        )
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, resultData: Intent?) {
        serviceConnectionManager.onVpnPermissionResult(resultCode == Activity.RESULT_OK)
    }

    override fun onStop() {
        Log.d("mullvad", "Stopping main activity")
        super.onStop()

        // NOTE: `super.onStop()` must be called before unbinding due to the fragment state handling
        // otherwise the fragments will believe there was an unexpected disconnect.
        serviceConnectionManager.unbind()

        deviceStateJob?.cancel()
    }

    override fun onDestroy() {
        serviceConnectionManager.onDestroy()
        super.onDestroy()
    }

    fun openAccount() {
        supportFragmentManager.beginTransaction().apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_bottom,
                R.anim.do_nothing,
                R.anim.do_nothing,
                R.anim.fragment_exit_to_bottom
            )
            replace(R.id.main_fragment, AccountFragment())
            addToBackStack(null)
            commitAllowingStateLoss()
        }
    }

    fun openSettings() {
        supportFragmentManager.beginTransaction().apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_bottom,
                R.anim.do_nothing,
                R.anim.do_nothing,
                R.anim.fragment_exit_to_bottom
            )
            replace(R.id.main_fragment, SettingsFragment())
            addToBackStack(null)
            commitAllowingStateLoss()
        }
    }

    private fun launchDeviceStateHandler(): Job {
        return lifecycleScope.launch {
            launch {
                deviceRepository.deviceState
                    .debounce {
                        // Debounce DeviceState.Unknown to delay view transitions during reconnect.
                        it.addDebounceForUnknownState(UNKNOWN_STATE_DEBOUNCE_DELAY_MILLISECONDS)
                    }
                    .collect { newState ->
                        if (newState != currentDeviceState)
                            when (newState) {
                                is DeviceState.Initial,
                                is DeviceState.Unknown -> openLaunchView()
                                is DeviceState.LoggedOut -> openLoginView()
                                is DeviceState.Revoked -> openRevokedView()
                                is DeviceState.LoggedIn -> {
                                    openLoggedInView(
                                        accountToken = newState.accountAndDevice.account_token,
                                        shouldDelayLogin =
                                            currentDeviceState is DeviceState.LoggedOut
                                    )
                                }
                            }
                        currentDeviceState = newState
                    }
            }

            lifecycleScope.launch {
                deviceRepository.deviceState
                    .filter { it is DeviceState.LoggedIn || it is DeviceState.LoggedOut }
                    .collect { loadChangelogComponent() }
            }
        }
    }

    private fun loadChangelogComponent() {
        findViewById<ComposeView>(R.id.compose_view).apply {
            setViewCompositionStrategy(ViewCompositionStrategy.DisposeOnDetachedFromWindow)
            setContent {
                val state = changelogViewModel.uiState.collectAsState().value
                if (state is ChangelogDialogUiState.Show) {
                    AppTheme {
                        ChangelogDialog(
                            changesList = state.changes,
                            version = BuildConfig.VERSION_NAME,
                            onDismiss = { changelogViewModel.dismissChangelogDialog() }
                        )
                    }
                }
            }
            changelogViewModel.refreshChangelogDialogUiState()
        }
    }

    @Suppress("DEPRECATION")
    private fun requestVpnPermission() {
        val intent = VpnService.prepare(this)

        startActivityForResult(intent, 0)
    }

    private fun openLaunchView() {
        supportFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, LoadingFragment())
            commitAllowingStateLoss()
        }
    }

    private fun openPrivacyDisclaimerFragment() {
        supportFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, PrivacyDisclaimerFragment())
            commitAllowingStateLoss()
        }
    }

    private suspend fun openLoggedInView(accountToken: String, shouldDelayLogin: Boolean) {
        val isNewAccount = accountToken == accountRepository.cachedCreatedAccount.value
        val isExpired = isNewAccount.not() && isExpired(LOGIN_AWAIT_EXPIRY_MILLIS)

        val fragment =
            when {
                isNewAccount -> WelcomeFragment()
                isExpired -> {
                    if (shouldDelayLogin) {
                        delay(LOGIN_DELAY_MILLIS)
                    }
                    OutOfTimeFragment()
                }
                else -> {
                    if (shouldDelayLogin) {
                        delay(LOGIN_DELAY_MILLIS)
                    }
                    ConnectFragment()
                }
            }

        supportFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, fragment)
            commitAllowingStateLoss()
        }
    }

    private suspend fun isExpired(timeoutMillis: Long): Boolean {
        return withTimeoutOrNull(timeoutMillis) {
            accountRepository.accountExpiryState
                .onSubscription { accountRepository.fetchAccountExpiry() }
                .filter { it is AccountExpiry.Available }
                .map { it.date()?.isBeforeNow }
                .first()
        }
            ?: false
    }

    private fun openLoginView() {
        clearBackStack()
        supportFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, LoginFragment())
            commitAllowingStateLoss()
        }
    }

    private fun openRevokedView() {
        clearBackStack()
        supportFragmentManager.beginTransaction().apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_right,
                R.anim.fragment_exit_to_left,
                R.anim.fragment_half_enter_from_left,
                R.anim.fragment_exit_to_right
            )
            replace(R.id.main_fragment, DeviceRevokedFragment())
            commitAllowingStateLoss()
        }
    }

    fun clearBackStack() {
        supportFragmentManager.apply {
            if (backStackEntryCount > 0) {
                val firstEntry = getBackStackEntryAt(0)
                popBackStack(firstEntry.id, FragmentManager.POP_BACK_STACK_INCLUSIVE)
            }
        }
    }

    private fun checkForNotificationPermission() {
        if (isNotificationPermissionGranted().not()) {
            getNotificationPermissionResource()?.let {
                requestNotificationPermissionLauncher.launch(it)
            }
        }
    }

    companion object {
        private const val LOGIN_DELAY_MILLIS = 1000L
        private const val LOGIN_AWAIT_EXPIRY_MILLIS = 1000L
    }
}

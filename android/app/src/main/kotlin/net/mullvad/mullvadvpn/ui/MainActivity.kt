package net.mullvad.mullvadvpn.ui

import android.app.Activity
import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.view.WindowCompat
import net.mullvad.mullvadvpn.compose.screen.MullvadApp
import net.mullvad.mullvadvpn.di.paymentModule
import net.mullvad.mullvadvpn.di.uiModule
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils.requestNotificationPermissionIfMissing
import net.mullvad.mullvadvpn.lib.endpoint.getApiEndpointConfigurationExtras
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import net.mullvad.mullvadvpn.viewmodel.NoDaemonViewModel
import org.koin.android.ext.android.getKoin
import org.koin.core.context.loadKoinModules

class MainActivity : ComponentActivity() {
    private val requestNotificationPermissionLauncher =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) {
            // NotificationManager.areNotificationsEnabled is used to check the state rather than
            // handling the callback value.
        }

    private lateinit var accountRepository: AccountRepository
    private lateinit var deviceRepository: DeviceRepository
    private lateinit var privacyDisclaimerRepository: PrivacyDisclaimerRepository
    private lateinit var serviceConnectionManager: ServiceConnectionManager
    private lateinit var changelogViewModel: ChangelogViewModel
    private lateinit var serviceConnectionViewModel: NoDaemonViewModel

    override fun onCreate(savedInstanceState: Bundle?) {
        loadKoinModules(listOf(uiModule, paymentModule))

        // Tell the system that we will draw behind the status bar and navigation bar
        WindowCompat.setDecorFitsSystemWindows(window, false)

        getKoin().apply {
            accountRepository = get()
            deviceRepository = get()
            privacyDisclaimerRepository = get()
            serviceConnectionManager = get()
            changelogViewModel = get()
            serviceConnectionViewModel = get()
        }
        lifecycle.addObserver(serviceConnectionViewModel)

        super.onCreate(savedInstanceState)

        setContent { AppTheme { MullvadApp() } }
    }

    fun initializeStateHandlerAndServiceConnection() {
        requestNotificationPermissionIfMissing(requestNotificationPermissionLauncher)
        serviceConnectionManager.bind(
            vpnPermissionRequestHandler = ::requestVpnPermission,
            apiEndpointConfiguration = intent?.getApiEndpointConfigurationExtras()
        )
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, resultData: Intent?) {
        serviceConnectionManager.onVpnPermissionResult(resultCode == Activity.RESULT_OK)
    }

    override fun onStart() {
        super.onStart()

        if (privacyDisclaimerRepository.hasAcceptedPrivacyDisclosure()) {
            initializeStateHandlerAndServiceConnection()
        }
    }

    override fun onStop() {
        Log.d("mullvad", "Stopping main activity")
        super.onStop()

        // NOTE: `super.onStop()` must be called before unbinding due to the fragment state handling
        // otherwise the fragments will believe there was an unexpected disconnect.
        serviceConnectionManager.unbind()
    }

    override fun onDestroy() {
        serviceConnectionManager.onDestroy()
        lifecycle.removeObserver(serviceConnectionViewModel)
        super.onDestroy()
    }

    @Suppress("DEPRECATION")
    private fun requestVpnPermission() {
        val intent = VpnService.prepare(this)

        startActivityForResult(intent, 0)
    }
}

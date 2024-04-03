package net.mullvad.mullvadvpn.ui

import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.collectAsState
import androidx.core.view.WindowCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.screen.MullvadApp
import net.mullvad.mullvadvpn.di.paymentModule
import net.mullvad.mullvadvpn.di.uiModule
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils.requestNotificationPermissionIfMissing
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.endpoint.getApiEndpointConfigurationExtras
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.viewmodel.NoDaemonViewModel
import org.koin.android.ext.android.getKoin
import org.koin.core.context.loadKoinModules

class MainActivity : ComponentActivity() {
    private val requestNotificationPermissionLauncher =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) {
            // NotificationManager.areNotificationsEnabled is used to check the state rather than
            // handling the callback value.
        }

    private lateinit var privacyDisclaimerRepository: PrivacyDisclaimerRepository
    private lateinit var serviceConnectionManager: ServiceConnectionManager
    private lateinit var noDaemonViewModel: NoDaemonViewModel
    private lateinit var managementService: ManagementService

    override fun onCreate(savedInstanceState: Bundle?) {
        loadKoinModules(listOf(uiModule, paymentModule))

        // Tell the system that we will draw behind the status bar and navigation bar
        WindowCompat.setDecorFitsSystemWindows(window, false)

        with(getKoin()) {
            privacyDisclaimerRepository = get()
            serviceConnectionManager = get()
            noDaemonViewModel = get()
            managementService = get()
        }
        lifecycle.addObserver(noDaemonViewModel)

        super.onCreate(savedInstanceState)

        setContent {
            AppTheme {
                MullvadApp()
                val currentState = managementService.connectionState.collectAsState()
                Text(
                    text = currentState.value.toString(),
                    style = MaterialTheme.typography.titleLarge
                )
            }
        }

        // This is to protect against tapjacking attacks
        window.decorView.filterTouchesWhenObscured = true

        // We use lifecycleScope here to get less start service in background exceptions
        // Se this article for more information:
        // https://medium.com/@lepicekmichal/android-background-service-without-hiccup-501e4479110f
        lifecycleScope.launch {
            repeatOnLifecycle(Lifecycle.State.STARTED) {
                if (privacyDisclaimerRepository.hasAcceptedPrivacyDisclosure()) {
                    startServiceSuspend()
                    startManagementService()
                }
            }
        }
    }

    fun startServiceSuspend() {
        requestNotificationPermissionIfMissing(requestNotificationPermissionLauncher)
        serviceConnectionManager.bind(
            // vpnPermissionRequestHandler = ::requestVpnPermission,
            apiEndpointConfiguration = intent?.getApiEndpointConfigurationExtras()
        )
    }

    suspend fun startManagementService() {
        delay(100)
        managementService.start()
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, resultData: Intent?) {
        // super call is needed for return value when opening file.
        super.onActivityResult(requestCode, resultCode, resultData)

        // Ensure we are responding to the correct request
        if (requestCode == REQUEST_VPN_PERMISSION_RESULT_CODE) {
//        serviceConnectionManager.onVpnPermissionResult(resultCode == Activity.RESULT_OK)
        }
    }

    override fun onStop() {
        Log.d("mullvad", "Stopping main activity")
        super.onStop()
        serviceConnectionManager.unbind()
    }

    override fun onDestroy() {
        serviceConnectionManager.onDestroy()
        lifecycle.removeObserver(noDaemonViewModel)
        super.onDestroy()
    }

    @Suppress("DEPRECATION")
    private fun requestVpnPermission() {
        val intent = VpnService.prepare(this)

        startActivityForResult(intent, REQUEST_VPN_PERMISSION_RESULT_CODE)
    }

    companion object {
        private const val REQUEST_VPN_PERMISSION_RESULT_CODE = 0
    }
}

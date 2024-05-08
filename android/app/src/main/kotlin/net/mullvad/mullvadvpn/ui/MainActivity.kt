package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.core.view.WindowCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.screen.MullvadApp
import net.mullvad.mullvadvpn.di.paymentModule
import net.mullvad.mullvadvpn.di.uiModule
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils.requestNotificationPermissionIfMissing
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.endpoint.getApiEndpointConfigurationExtras
import net.mullvad.mullvadvpn.lib.intent.IntentProvider
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
    private lateinit var intentProvider: IntentProvider

    override fun onCreate(savedInstanceState: Bundle?) {
        Log.d("mullvad", "onCreate")
        loadKoinModules(listOf(uiModule, paymentModule))

        // Tell the system that we will draw behind the status bar and navigation bar
        WindowCompat.setDecorFitsSystemWindows(window, false)

        with(getKoin()) {
            privacyDisclaimerRepository = get()
            serviceConnectionManager = get()
            noDaemonViewModel = get()
            managementService = get()
            intentProvider = get()
        }
        lifecycle.addObserver(noDaemonViewModel)

        super.onCreate(savedInstanceState)

        if (savedInstanceState == null) {
            intentProvider.setStartIntent(intent)
        }

        setContent {
            AppTheme {
                MullvadApp()
                val currentState = managementService.connectionState.collectAsStateWithLifecycle()
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
                Log.d("mullvad", "repeatOnLifecycle STARTED")
                if (privacyDisclaimerRepository.hasAcceptedPrivacyDisclosure()) {
                    startServiceSuspend()
                }
            }
        }
    }

    //    override fun onNewIntent(intent: Intent?) {
    //        super.onNewIntent(intent)
    //        intentProvider.setStartIntent(intent)
    //    }

    fun startServiceSuspend() {
        requestNotificationPermissionIfMissing(requestNotificationPermissionLauncher)
        serviceConnectionManager.bind(
            // vpnPermissionRequestHandler = ::requestVpnPermission,
            apiEndpointConfiguration = intent?.getApiEndpointConfigurationExtras()
        )
    }

    override fun onStart() {
        super.onStart()
        Log.d("mullvad", "onStart")
    }

    override fun onResume() {
        super.onResume()
        Log.d("mullvad", "onResume")
    }

    override fun onRestart() {
        super.onRestart()
        Log.d("mullvad", "onRestart")
    }

    override fun onStop() {
        super.onStop()
        Log.d("mullvad", "onStop")
        serviceConnectionManager.unbind()
    }

    override fun onDestroy() {
        Log.d("mullvad", "onDestroy")
        serviceConnectionManager.onDestroy()
        lifecycle.removeObserver(noDaemonViewModel)
        super.onDestroy()
    }
}

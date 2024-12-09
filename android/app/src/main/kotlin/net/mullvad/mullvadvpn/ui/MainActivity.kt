package net.mullvad.mullvadvpn.ui

import android.content.Intent
import android.graphics.Color
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.SystemBarStyle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.screen.MullvadApp
import net.mullvad.mullvadvpn.di.paymentModule
import net.mullvad.mullvadvpn.di.uiModule
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils.requestNotificationPermissionIfMissing
import net.mullvad.mullvadvpn.lib.daemon.grpc.GrpcConnectivityState
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.intent.IntentProvider
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository
import net.mullvad.mullvadvpn.repository.SplashCompleteRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.viewmodel.NoDaemonViewModel
import org.koin.android.ext.android.inject
import org.koin.android.scope.AndroidScopeComponent
import org.koin.androidx.scope.activityScope
import org.koin.core.context.loadKoinModules

class MainActivity : ComponentActivity(), AndroidScopeComponent {
    override val scope by activityScope()

    private val requestNotificationPermissionLauncher =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) {
            // NotificationManager.areNotificationsEnabled is used to check the state rather than
            // handling the callback value.
        }

    private val intentProvider by inject<IntentProvider>()
    private val noDaemonViewModel by inject<NoDaemonViewModel>()
    private val privacyDisclaimerRepository by inject<PrivacyDisclaimerRepository>()
    private val serviceConnectionManager by inject<ServiceConnectionManager>()
    private val splashCompleteRepository by inject<SplashCompleteRepository>()
    private val managementService by inject<ManagementService>()

    private var isReadyNextDraw: Boolean = false

    override fun onCreate(savedInstanceState: Bundle?) {
        loadKoinModules(listOf(uiModule, paymentModule))

        lifecycle.addObserver(noDaemonViewModel)

        installSplashScreen().setKeepOnScreenCondition {
            val isReady = isReadyNextDraw
            isReadyNextDraw = splashCompleteRepository.isSplashComplete()
            !isReady
        }

        enableEdgeToEdge(
            statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
            navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
        )

        super.onCreate(savedInstanceState)

        // Needs to be before set content since we want to access the intent in compose
        if (savedInstanceState == null) {
            intentProvider.setStartIntent(intent)
        }
        setContent { AppTheme { MullvadApp() } }

        // This is to protect against tapjacking attacks
        window.decorView.filterTouchesWhenObscured = true

        // We use lifecycleScope here to get less start service in background exceptions
        // Se this article for more information:
        // https://medium.com/@lepicekmichal/android-background-service-without-hiccup-501e4479110f
        lifecycleScope.launch {
            repeatOnLifecycle(Lifecycle.State.STARTED) {
                if (privacyDisclaimerRepository.hasAcceptedPrivacyDisclosure()) {
                    bindService()
                }
            }
        }
    }

    override fun onRestoreInstanceState(savedInstanceState: Bundle) {
        super.onRestoreInstanceState(savedInstanceState)
        lifecycleScope.launch {
            if (privacyDisclaimerRepository.hasAcceptedPrivacyDisclosure()) {
                // If service is to be started wait for it to be connected before dismissing Splash
                // screen
                managementService.connectionState
                    .filter { it is GrpcConnectivityState.Ready }
                    .first()
            }
            splashCompleteRepository.onSplashCompleted()
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        intentProvider.setStartIntent(intent)
    }

    fun bindService() {
        requestNotificationPermissionIfMissing(requestNotificationPermissionLauncher)
        serviceConnectionManager.bind()
    }

    override fun onStop() {
        super.onStop()
        if (privacyDisclaimerRepository.hasAcceptedPrivacyDisclosure()) {
            serviceConnectionManager.unbind()
        }
    }

    override fun onDestroy() {
        lifecycle.removeObserver(noDaemonViewModel)
        super.onDestroy()
    }
}

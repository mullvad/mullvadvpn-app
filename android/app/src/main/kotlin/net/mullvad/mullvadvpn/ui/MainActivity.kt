package net.mullvad.mullvadvpn.ui

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.view.WindowCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import co.touchlab.kermit.Logger
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.screen.MullvadApp
import net.mullvad.mullvadvpn.di.paymentModule
import net.mullvad.mullvadvpn.di.uiModule
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils.requestNotificationPermissionIfMissing
import net.mullvad.mullvadvpn.lib.intent.IntentProvider
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.viewmodel.NoDaemonViewModel
import org.koin.android.scope.AndroidScopeComponent
import org.koin.androidx.scope.activityScope
import org.koin.androidx.scope.createActivityScope
import org.koin.core.context.loadKoinModules

class MainActivity : ComponentActivity(), AndroidScopeComponent {
    override val scope by activityScope()

    private val requestNotificationPermissionLauncher =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) {
            // NotificationManager.areNotificationsEnabled is used to check the state rather than
            // handling the callback value.
        }

    private lateinit var privacyDisclaimerRepository: PrivacyDisclaimerRepository
    private lateinit var serviceConnectionManager: ServiceConnectionManager
    private lateinit var noDaemonViewModel: NoDaemonViewModel
    private lateinit var intentProvider: IntentProvider

    override fun onCreate(savedInstanceState: Bundle?) {
        loadKoinModules(listOf(uiModule, paymentModule))

        // Tell the system that we will draw behind the status bar and navigation bar
        WindowCompat.setDecorFitsSystemWindows(window, false)

        with(createActivityScope()) {
            privacyDisclaimerRepository = get()
            serviceConnectionManager = get()
            noDaemonViewModel = get()
            intentProvider = get()
        }
        lifecycle.addObserver(noDaemonViewModel)

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
                Logger.d("MainActivity $this: STARTED")
                if (privacyDisclaimerRepository.hasAcceptedPrivacyDisclosure()) {
                    bindService()
                }
            }
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

    override fun onStart() {
        super.onStart()
        Logger.d("MainActivity $this: onStart")
    }

    override fun onStop() {
        super.onStop()
        Logger.d("MainActivity $this: onStop")
        serviceConnectionManager.unbind()
    }

    override fun onDestroy() {
        lifecycle.removeObserver(noDaemonViewModel)
        super.onDestroy()
    }
}

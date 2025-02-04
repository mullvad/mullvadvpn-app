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
import androidx.core.util.Consumer
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import arrow.core.merge
import co.touchlab.kermit.Logger
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.screen.MullvadApp
import net.mullvad.mullvadvpn.compose.util.CreateVpnProfile
import net.mullvad.mullvadvpn.di.paymentModule
import net.mullvad.mullvadvpn.di.uiModule
import net.mullvad.mullvadvpn.lib.common.constant.KEY_REQUEST_VPN_PROFILE
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils.requestNotificationPermissionIfMissing
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import net.mullvad.mullvadvpn.lib.daemon.grpc.GrpcConnectivityState
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointFromIntentHolder
import net.mullvad.mullvadvpn.lib.endpoint.getApiEndpointConfigurationExtras
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.mullvadvpn.lib.model.Prepared
import net.mullvad.mullvadvpn.lib.model.modelModule
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.repository.SplashCompleteRepository
import net.mullvad.mullvadvpn.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.viewmodel.MullvadAppViewModel
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
    private val launchVpnPermission =
        registerForActivityResult(CreateVpnProfile()) { _ -> mullvadAppViewModel.connect() }

    private val apiEndpointFromIntentHolder by inject<ApiEndpointFromIntentHolder>()
    private val mullvadAppViewModel by inject<MullvadAppViewModel>()
    private val userPreferencesRepository by inject<UserPreferencesRepository>()
    private val serviceConnectionManager by inject<ServiceConnectionManager>()
    private val splashCompleteRepository by inject<SplashCompleteRepository>()
    private val managementService by inject<ManagementService>()

    private var isReadyNextDraw: Boolean = false

    override fun onCreate(savedInstanceState: Bundle?) {
        loadKoinModules(listOf(uiModule, paymentModule, modelModule))

        lifecycle.addObserver(mullvadAppViewModel)

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

        setContent { AppTheme { MullvadApp() } }

        // This is to protect against tapjacking attacks
        window.decorView.rootView.filterTouchesWhenObscured = true

        // Needs to be before we start the service, since we need to access the intent there
        lifecycleScope.launch { intents().collect(::handleIntent) }

        // We use lifecycleScope here to get less start service in background exceptions
        // Se this article for more information:
        // https://medium.com/@lepicekmichal/android-background-service-without-hiccup-501e4479110f
        lifecycleScope.launch {
            repeatOnLifecycle(Lifecycle.State.STARTED) {
                if (userPreferencesRepository.preferences().isPrivacyDisclosureAccepted) {
                    bindService()
                }
            }
        }
    }

    override fun onRestoreInstanceState(savedInstanceState: Bundle) {
        super.onRestoreInstanceState(savedInstanceState)
        lifecycleScope.launch {
            if (userPreferencesRepository.preferences().isPrivacyDisclosureAccepted) {
                // If service is to be started wait for it to be connected before dismissing Splash
                // screen
                managementService.connectionState
                    .filter { it is GrpcConnectivityState.Ready }
                    .first()
            }
            splashCompleteRepository.onSplashCompleted()
        }
    }

    fun bindService() {
        requestNotificationPermissionIfMissing(requestNotificationPermissionLauncher)
        serviceConnectionManager.bind()
    }

    override fun onStop() {
        super.onStop()
        if (serviceConnectionManager.connectionState.value == ServiceConnectionState.Bound) {
            serviceConnectionManager.unbind()
        }
    }

    override fun onDestroy() {
        lifecycle.removeObserver(mullvadAppViewModel)
        super.onDestroy()
    }

    private fun handleIntent(intent: Intent) {
        when (val action = intent.action) {
            Intent.ACTION_MAIN ->
                apiEndpointFromIntentHolder.setApiEndpointOverride(
                    intent.getApiEndpointConfigurationExtras()
                )
            KEY_REQUEST_VPN_PROFILE -> handleRequestVpnProfileIntent()
            else -> Logger.w("Unhandled intent action: $action")
        }
    }

    private fun handleRequestVpnProfileIntent() {
        val prepareResult = prepareVpnSafe().merge()
        when (prepareResult) {
            is PrepareError.NotPrepared -> launchVpnPermission.launch(prepareResult.prepareIntent)
            // If legacy or other always on connect at let daemon generate a error state
            is PrepareError.OtherLegacyAlwaysOnVpn,
            is PrepareError.OtherAlwaysOnApp,
            Prepared -> mullvadAppViewModel.connect()
        }
    }

    private fun ComponentActivity.intents() =
        callbackFlow<Intent> {
            send(intent)

            val listener = Consumer<Intent> { intent -> trySend(intent) }

            addOnNewIntentListener(listener)

            awaitClose { removeOnNewIntentListener(listener) }
        }
}

package net.mullvad.mullvadvpn.service

import android.app.KeyguardManager
import android.content.Intent
import android.os.Binder
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.content.getSystemService
import androidx.lifecycle.lifecycleScope
import arrow.atomic.AtomicInt
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_DISCONNECT_ACTION
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.lib.endpoint.getApiEndpointConfigurationExtras
import net.mullvad.mullvadvpn.lib.intent.IntentProvider
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.service.di.apiEndpointModule
import net.mullvad.mullvadvpn.service.di.vpnServiceModule
import net.mullvad.mullvadvpn.service.notifications.ChannelFactory
import net.mullvad.mullvadvpn.service.notifications.ForegroundNotificationManager
import net.mullvad.mullvadvpn.service.notifications.NotificationManager
import net.mullvad.mullvadvpn.service.notifications.ShouldBeOnForegroundProvider
import net.mullvad.talpid.TalpidVpnService
import org.koin.android.ext.android.getKoin
import org.koin.core.context.loadKoinModules

class MullvadVpnService : TalpidVpnService(), ShouldBeOnForegroundProvider {
    private val _shouldBeOnForeground = MutableStateFlow(false)
    override val shouldBeOnForeground: StateFlow<Boolean> = _shouldBeOnForeground

    private lateinit var keyguardManager: KeyguardManager
    private lateinit var daemonInstance: MullvadDaemon

    private lateinit var apiEndpointConfiguration: ApiEndpointConfiguration
    private lateinit var managementService: ManagementService
    private lateinit var migrateSplitTunneling: MigrateSplitTunneling
    private lateinit var intentProvider: IntentProvider
    private lateinit var connectionProxy: ConnectionProxy

    private lateinit var foregroundNotificationHandler: ForegroundNotificationManager

    private val bindCount = AtomicInt()

    override fun onCreate() {
        super.onCreate()

        loadKoinModules(listOf(vpnServiceModule, apiEndpointModule))
        with(getKoin()) {
            // Needed to create all the notification channels
            get<ChannelFactory>()

            managementService = get()

            foregroundNotificationHandler =
                ForegroundNotificationManager(this@MullvadVpnService, get(), lifecycleScope)
            get<NotificationManager>()

            apiEndpointConfiguration = get()
            migrateSplitTunneling = get()
            intentProvider = get()
            connectionProxy = get()
        }

        keyguardManager = getSystemService<KeyguardManager>()!!

        lifecycleScope.launch { foregroundNotificationHandler.start(this@MullvadVpnService) }

        // TODO We should avoid lifecycleScope.launch (current needed due to InetSocketAddress
        // with intent from API)
        lifecycleScope.launch(context = Dispatchers.IO) {
            managementService.start()
            daemonInstance =
                MullvadDaemon(
                    vpnService = this@MullvadVpnService,
                    apiEndpointConfiguration =
                        intentProvider.getLatestIntent()?.getApiEndpointConfigurationExtras()
                            ?: apiEndpointConfiguration,
                    migrateSplitTunneling = migrateSplitTunneling
                )
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.d(TAG, "onStartCommand (intent=$intent, flags=$flags, startId=$startId)")
        Log.d(TAG, "intent action=${intent?.action}")

        val startResult = super.onStartCommand(intent, flags, startId)

        // Always promote to foreground if connect/disconnect actions are provided to mitigate cases
        // where the service would potentially otherwise be too slow running `startForeground`.
        Log.d(TAG, "Intent Action: ${intent?.action}")
        when {
            keyguardManager.isKeyguardLocked -> {
                Log.d(TAG, "Keyguard is locked, ignoring command")
            }
            intent.isFromSystem() || intent?.action == KEY_CONNECT_ACTION -> {
                _shouldBeOnForeground.update { true }
                lifecycleScope.launch { connectionProxy.connectWithoutPermissionCheck() }
            }
            intent?.action == KEY_DISCONNECT_ACTION -> {
                lifecycleScope.launch { connectionProxy.disconnect() }
            }
        }

        return startResult
    }

    override fun onBind(intent: Intent?): IBinder {
        bindCount.incrementAndGet()
        Log.d(TAG, "onBind: $intent")

        if (intent.isFromSystem()) {
            Log.d(TAG, "onBind from VPN_SERVICE_CLASS")
            _shouldBeOnForeground.update { true }
        }
        return super.onBind(intent)
            ?: if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                Binder(this.toString())
            } else {
                Binder()
            }
    }

    override fun onRevoke() {
        runBlocking { connectionProxy.disconnect() }
    }

    override fun onUnbind(intent: Intent): Boolean {
        val count = bindCount.decrementAndGet()

        // Foreground?
        if (intent.isFromSystem()) {
            _shouldBeOnForeground.update { false }
        }

        if (count == 0) {
            Log.d(TAG, "No one bound to the service, stopSelf()")
            lifecycleScope.launch {
                Log.d(TAG, "Waiting for disconnected state")
                // TODO This needs reworking, we should not wait for the disconnected state, what we
                // want is the notification of disconnected to go out before we start shutting down
                connectionProxy.tunnelState
                    .filter {
                        it is TunnelState.Disconnected ||
                            (it is TunnelState.Error && !it.errorState.isBlocking)
                    }
                    .first()

                if (bindCount.get() == 0) {
                    Log.d(TAG, "Stopping service")
                    stopSelf()
                }
            }
        }
        return false
    }

    override fun onDestroy() {
        managementService.stop()

        // Shutting down the daemon gracefully
        runBlocking { daemonInstance.shutdown() }
        super.onDestroy()
    }

    private fun Intent?.isFromSystem(): Boolean {
        return this?.action == SERVICE_INTERFACE
    }

    companion object {
        private const val TAG = "MullvadVpnService"

        init {
            System.loadLibrary("mullvad_jni")
        }
    }
}

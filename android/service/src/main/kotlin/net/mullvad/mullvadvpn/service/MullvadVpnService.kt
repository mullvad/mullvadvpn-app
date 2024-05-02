package net.mullvad.mullvadvpn.service

import android.annotation.SuppressLint
import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import android.os.Binder
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.lifecycle.lifecycleScope
import arrow.atomic.AtomicInt
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterIsInstance
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
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.repository.MigrateSplitTunnelingRepository
import net.mullvad.mullvadvpn.service.di.apiEndpointModule
import net.mullvad.mullvadvpn.service.di.vpnServiceModule
import net.mullvad.mullvadvpn.service.notifications.ChannelFactory
import net.mullvad.mullvadvpn.service.notifications.ForegroundNotificationManager
import net.mullvad.mullvadvpn.service.notifications.NotificationManager
import net.mullvad.mullvadvpn.service.notifications.ShouldBeOnForegroundProvider
import net.mullvad.talpid.TalpidVpnService
import org.koin.android.ext.android.get
import org.koin.android.ext.android.getKoin
import org.koin.core.context.loadKoinModules

class MullvadVpnService : TalpidVpnService(), ShouldBeOnForegroundProvider {
    private val _shouldBeOnForeground = MutableStateFlow(false)
    override val shouldBeOnForeground: StateFlow<Boolean> = _shouldBeOnForeground

    private lateinit var keyguardManager: KeyguardManager
    private lateinit var daemonInstance: MullvadDaemon

    private lateinit var apiEndpointConfiguration: ApiEndpointConfiguration
    private lateinit var managementService: ManagementService
    private lateinit var migrateSplitTunnelingRepository: MigrateSplitTunnelingRepository
    private lateinit var intentProvider: IntentProvider

    private lateinit var foregroundNotificationHandler: ForegroundNotificationManager

    private val bindCount = AtomicInt()
    // Suppressing since the tunnel state pref should be writted immediately.
    @SuppressLint("ApplySharedPref")
    override fun onCreate() {
        super.onCreate()
        // TODO We should probably run things in the background here?
        // TODO Remove run blocking?
        Log.d(TAG, "onCreate")

        loadKoinModules(listOf(vpnServiceModule, apiEndpointModule))
        with(getKoin()) {
            get<ChannelFactory>()
            managementService = get()

            foregroundNotificationHandler =
                ForegroundNotificationManager(this@MullvadVpnService, get(), lifecycleScope)
            get<NotificationManager>()
        }

        lifecycleScope.launch { managementService.start() }
        lifecycleScope.launch { foregroundNotificationHandler.start(this@MullvadVpnService) }

        keyguardManager = getSystemService(Context.KEYGUARD_SERVICE) as KeyguardManager

        apiEndpointConfiguration = get()
        migrateSplitTunnelingRepository = get()
        intentProvider = get()
        lifecycleScope.launch(context = Dispatchers.IO) {
            daemonInstance =
                MullvadDaemon(
                    vpnService = this@MullvadVpnService,
                    apiEndpointConfiguration =
                        intentProvider.getLatestIntent()?.getApiEndpointConfigurationExtras()
                            ?: apiEndpointConfiguration,
                    migrateSplitTunnelingRepository = migrateSplitTunnelingRepository
                )
        }

        //        endpoint.splitTunneling.onChange.subscribe(this@MullvadVpnService) { excludedApps
        // ->
        //            disallowedApps = excludedApps
        //            markTunAsStale()
        //            connectionProxy.reconnect()
        //        }

        //        notificationManager =
        //            ForegroundNotificationManager(this, connectionProxy,
        // daemonInstance.intermittentDaemon)

        //        accountExpiryNotification =
        //            AccountExpiryNotification(
        //                this,
        //                daemonInstance.intermittentDaemon,
        //                //endpoint.accountCache
        //            )
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Log.d(TAG, "onStartCommand (intent=$intent, flags=$flags, startId=$startId)")
        Log.d(TAG, "intent action=${intent?.action}")

        val startResult = super.onStartCommand(intent, flags, startId)

        // Always promote to foreground if connect/disconnect actions are provided to mitigate cases
        // where the service would potentially otherwise be too slow running `startForeground`.
        Log.d(TAG, "Intent Action: ${intent?.action}")
        when (intent?.action) {
            KEY_CONNECT_ACTION -> {
                _shouldBeOnForeground.update { true }
                runBlocking { managementService.connect() }
            }
            KEY_DISCONNECT_ACTION -> {
                _shouldBeOnForeground.update { true }
                runBlocking { managementService.disconnect() }
                _shouldBeOnForeground.update { false }
            }
        }

        //        if (!keyguardManager.isDeviceLocked) {
        //            val action = intent?.action
        //
        //            if (action == SERVICE_INTERFACE || action == KEY_CONNECT_ACTION) {
        //                pendingAction = PendingAction.Connect
        //            } else if (action == KEY_DISCONNECT_ACTION) {
        //                pendingAction = PendingAction.Disconnect
        //            }
        //        }

        Log.d(TAG, "onStartCommand result: $startResult")
        return startResult
    }

    override fun onBind(intent: Intent?): IBinder {
        bindCount.incrementAndGet()
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
        Log.d(TAG, "onRevoke")
        runBlocking { managementService.disconnect() }
        super.onRevoke()
    }

    override fun onUnbind(intent: Intent): Boolean {
        val bindCount = bindCount.decrementAndGet()

        Log.d(TAG, "onUnbind1 $intent")
        // Foreground?

        if (intent.isFromSystem()) {
            Log.d(TAG, "onUnbind from VPN_SERVICE_CLASS")
            _shouldBeOnForeground.update { false }
        }

        val currentTunnelState = runBlocking { managementService.tunnelState.first() }
        Log.d(TAG, "onUnbind currentTunnelState: $currentTunnelState")

        if (bindCount == 0) {
            Log.d(TAG, "No one bound to the service, stopSelf()")
            lifecycleScope.launch {
                managementService.tunnelState.filterIsInstance<TunnelState.Disconnected>().first()
                stopSelf()
            }
        }

        //        val shouldKill =
        //            if (intent.action == VPN_SERVICE_CLASS) {
        //                Log.d(TAG, "onUnbind from VPN_SERVICE_CLASS")
        //                runBlocking { managementService.disconnect() }
        //                true
        //            } else false
        //        if (shouldKill) {
        //            Log.d(TAG, "onUnbind stopSelf()")
        //            stopSelf()
        //        }
        return false
    }

    override fun onDestroy() {
        Log.d(TAG, "onDestroy")
        daemonInstance.onDestroy()
        super.onDestroy()
    }

    override fun onTaskRemoved(rootIntent: Intent?) {
        Log.d(TAG, "onTaskRemoved (rootIntent=$rootIntent)")
    }

    companion object {
        private const val TAG = "MullvadVpnService"

        init {
            System.loadLibrary("mullvad_jni")
        }
    }

    private fun Intent?.isFromSystem(): Boolean {
        return this?.action == SERVICE_INTERFACE
    }
}

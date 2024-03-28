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
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.sync.Semaphore
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_DISCONNECT_ACTION
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.lib.endpoint.getApiEndpointConfigurationExtras
import net.mullvad.mullvadvpn.service.di.apiEndpointModule
import net.mullvad.mullvadvpn.service.di.vpnServiceModule
import net.mullvad.mullvadvpn.service.notifications.AccountExpiryNotification
import net.mullvad.talpid.TalpidVpnService
import org.koin.android.ext.android.get
import org.koin.android.ext.android.getKoin
import org.koin.core.context.loadKoinModules

class MullvadVpnService : TalpidVpnService() {

    private lateinit var accountExpiryNotification: AccountExpiryNotification
    private lateinit var keyguardManager: KeyguardManager
    private lateinit var notificationManager: ForegroundNotificationManager
    private lateinit var daemonInstance: MullvadDaemon

    private lateinit var apiEndpointConfiguration: ApiEndpointConfiguration
    private lateinit var managementService: ManagementService

    // Suppressing since the tunnel state pref should be writted immediately.
    @SuppressLint("ApplySharedPref")
    override fun onCreate() {
        super.onCreate()
        Log.d(TAG, "onCreate")

        loadKoinModules(listOf(vpnServiceModule, apiEndpointModule))
        with(getKoin()) { managementService = get() }

        lifecycleScope.launch { managementService.start() }
        notificationManager = ForegroundNotificationManager(this, managementService)

        keyguardManager = getSystemService(Context.KEYGUARD_SERVICE) as KeyguardManager

        apiEndpointConfiguration = get()
        daemonInstance = MullvadDaemon(this, apiEndpointConfiguration)
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

        val intentProvidedConfiguration =
            if (BuildConfig.DEBUG) {
                intent?.getApiEndpointConfigurationExtras()
            } else {
                null
            }


        //        daemonInstance.apply {
        //            intermittentDaemon.registerListener(this@MullvadVpnService) { daemon ->
        //                handleDaemonInstance(daemon)
        //            }
        //
        //            start(apiEndpointConfiguration)
        //        }

        val startResult = super.onStartCommand(intent, flags, startId)

        // Always promote to foreground if connect/disconnect actions are provided to mitigate cases
        // where the service would potentially otherwise be too slow running `startForeground`.
        if (intent?.action == KEY_CONNECT_ACTION || intent?.action == KEY_DISCONNECT_ACTION) {
            notificationManager.showOnForeground()
        }

        // notificationManager.updateNotification()

        // Service was started from system, e.g Always-on VPN enabled
        if (intent?.action == SERVICE_INTERFACE) {
            notificationManager.showOnForeground()
            runBlocking { managementService.connect() }
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

    val sa = Semaphore(10)

    override fun onBind(intent: Intent?): IBinder {
        Log.d(TAG, "onBind $intent")
        runBlocking { sa.acquire() }
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
        sa.release()

        Log.d(TAG, "onUnbind1 $intent")
        // Foreground?
        val currentTunnelState = runBlocking { managementService.tunnelState.first() }
        Log.d(TAG, "onUnbind currentTunnelState: $currentTunnelState")

        if (sa.availablePermits == 10) {
            Log.d(TAG, "onUnbind stopSelf()")
            stopSelf()
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
}

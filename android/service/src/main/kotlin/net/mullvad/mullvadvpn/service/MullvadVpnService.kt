package net.mullvad.mullvadvpn.service

import android.annotation.SuppressLint
import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import android.os.Binder
import android.os.Build
import android.os.IBinder
import android.util.Log
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Job
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_DISCONNECT_ACTION
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.lib.endpoint.getApiEndpointConfigurationExtras
import net.mullvad.mullvadvpn.service.di.apiEndpointModule
import net.mullvad.mullvadvpn.service.di.vpnServiceModule
import net.mullvad.mullvadvpn.service.notifications.AccountExpiryNotification
import net.mullvad.talpid.TalpidVpnService
import org.koin.android.ext.android.get
import org.koin.core.context.loadKoinModules

class MullvadVpnService : TalpidVpnService() {

    private enum class PendingAction {
        Connect,
        Disconnect,
    }

    private enum class State {
        Running,
        Stopping,
        Stopped,
    }

    private var state = State.Running

    private var setUpDaemonJob: Job? = null

    private lateinit var accountExpiryNotification: AccountExpiryNotification
    private lateinit var keyguardManager: KeyguardManager
    private lateinit var notificationManager: ForegroundNotificationManager
    private lateinit var daemonInstance: MullvadDaemon

    private var pendingAction by
        observable<PendingAction?>(null) { _, _, _ ->
            // endpoint.settingsListener.settings?.let { settings -> handlePendingAction(settings) }
        }

    private lateinit var apiEndpointConfiguration: ApiEndpointConfiguration

    // Suppressing since the tunnel state pref should be writted immediately.
    @SuppressLint("ApplySharedPref")
    override fun onCreate() {
        super.onCreate()
        Log.d(TAG, "Initializing service")

        loadKoinModules(listOf(vpnServiceModule, apiEndpointModule))

        keyguardManager = getSystemService(Context.KEYGUARD_SERVICE) as KeyguardManager

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
        Log.d(TAG, "Starting service")

        val intentProvidedConfiguration =
            if (BuildConfig.DEBUG) {
                intent?.getApiEndpointConfigurationExtras()
            } else {
                null
            }

        apiEndpointConfiguration = intentProvidedConfiguration ?: get()
        daemonInstance = MullvadDaemon(this, apiEndpointConfiguration)

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

        if (!keyguardManager.isDeviceLocked) {
            val action = intent?.action

            if (action == SERVICE_INTERFACE || action == KEY_CONNECT_ACTION) {
                pendingAction = PendingAction.Connect
            } else if (action == KEY_DISCONNECT_ACTION) {
                pendingAction = PendingAction.Disconnect
            }
        }

        if (state == State.Stopping) {
            //            restart()
        }

        Log.d(TAG, "onStartCommand result: $startResult")
        return startResult
    }

    override fun onBind(intent: Intent?): IBinder {
        val binder =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                Binder(this.toString())
            } else {
                Binder()
            }
        Log.d(TAG, "onBind: binder=$binder")
        return binder
    }

    override fun onRevoke() {
        super.onRevoke()
        Log.d(TAG, "onRevoke")
        // Todo implement disconnect of VPN
    }

    override fun onUnbind(intent: Intent): Boolean {
        Log.d(TAG, "onUnbind")
        // TODO implement shutdown
        return true
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
        private const val TAG = "mullvad"

        init {
            System.loadLibrary("mullvad_jni")
        }
    }
}

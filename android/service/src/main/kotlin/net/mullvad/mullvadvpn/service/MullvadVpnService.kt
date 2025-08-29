package net.mullvad.mullvadvpn.service

import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import android.os.Binder
import android.os.Build
import android.os.IBinder
import androidx.core.content.getSystemService
import androidx.lifecycle.lifecycleScope
import arrow.atomic.AtomicInt
import co.touchlab.kermit.Logger
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_DISCONNECT_ACTION
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointFromIntentHolder
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.service.di.vpnServiceModule
import net.mullvad.mullvadvpn.service.migration.MigrateSplitTunneling
import net.mullvad.mullvadvpn.service.notifications.ForegroundNotificationManager
import net.mullvad.mullvadvpn.service.notifications.NotificationChannelFactory
import net.mullvad.mullvadvpn.service.notifications.NotificationManager
import net.mullvad.mullvadvpn.service.util.extractAndOverwriteIfAssetMoreRecent
import net.mullvad.talpid.TalpidVpnService
import org.koin.android.ext.android.getKoin
import org.koin.core.context.loadKoinModules

private const val RELAY_LIST_ASSET_NAME = "relays.json"

class MullvadVpnService : TalpidVpnService() {

    private lateinit var keyguardManager: KeyguardManager

    private lateinit var managementService: ManagementService
    private lateinit var migrateSplitTunneling: MigrateSplitTunneling
    private lateinit var apiEndpointFromIntentHolder: ApiEndpointFromIntentHolder
    private lateinit var connectionProxy: ConnectionProxy
    private lateinit var daemonConfig: DaemonConfig

    private lateinit var foregroundNotificationHandler: ForegroundNotificationManager

    // Count number of binds to know if the service is needed. If user actively using the VPN, a
    // bind from the system, should always be present.
    private val bindCount = AtomicInt()

    override fun onCreate() {
        super.onCreate()
        Logger.i("MullvadVpnService: onCreate")

        loadKoinModules(listOf(vpnServiceModule))
        with(getKoin()) {
            // Needed to create all the notification channels
            get<NotificationChannelFactory>()

            managementService = get()

            foregroundNotificationHandler =
                ForegroundNotificationManager(this@MullvadVpnService, get())
            get<NotificationManager>()

            daemonConfig = get()
            migrateSplitTunneling = get()
            apiEndpointFromIntentHolder = get()
            connectionProxy = get()
        }

        keyguardManager = getSystemService<KeyguardManager>()!!

        prepareFiles()
        migrateSplitTunneling.migrate()

        // If it is a debug build and we have an api override in the intent, use it
        // This is for injecting hostname and port for our mock api tests
        val intentApiOverride = apiEndpointFromIntentHolder.apiEndpointOverride
        val updatedConfig =
            if (BuildConfig.DEBUG && intentApiOverride != null) {
                daemonConfig.copy(apiEndpointOverride = intentApiOverride)
            } else {
                daemonConfig
            }
        Logger.i("Start daemon")
        startDaemon(updatedConfig)

        Logger.i("Start management service")
        managementService.start()

        lifecycleScope.launch {
            // If the service is started with a connect command and a non-blocking error occur (e.g.
            // unable to start the tunnel) then the service is demoted from foreground.
            managementService.tunnelState
                .filterIsInstance<TunnelState.Error>()
                .filter { !it.errorState.isBlocking }
                .collect { foregroundNotificationHandler.stopForeground() }
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        Logger.i(
            "onStartCommand (intent=$intent, action=${intent?.action}, flags=$flags, startId=$startId)"
        )

        val startResult = super.onStartCommand(intent, flags, startId)

        // Always promote to foreground if connect/disconnect actions are provided to mitigate cases
        // where the service would potentially otherwise be too slow running `startForeground`.
        when {
            keyguardManager.isKeyguardLocked -> {
                Logger.i("Keyguard is locked, ignoring command")
            }

            intent.isFromSystem() || intent?.action == KEY_CONNECT_ACTION -> {
                foregroundNotificationHandler.startForeground()
                lifecycleScope.launch { connectionProxy.connectWithoutPermissionCheck() }
            }

            intent?.action == KEY_DISCONNECT_ACTION -> {
                // MullvadTileService might have launched this service with the expectancy of it
                // being foreground, thus it must go into foreground to please the android system
                // requirements.
                foregroundNotificationHandler.startForeground()
                lifecycleScope.launch { connectionProxy.disconnect() }

                // If disconnect intent is received and no one is using this service, simply stop
                // foreground and let system stop service when it deems it not to be necessary.
                if (bindCount.get() == 0) {
                    foregroundNotificationHandler.stopForeground()
                }
            }
        }

        return startResult
    }

    override fun onBind(intent: Intent?): IBinder {
        val count = bindCount.incrementAndGet()
        Logger.i("onBind: $intent, bindCount: $count")

        if (intent.isFromSystem()) {
            Logger.i("onBind was from system")
            foregroundNotificationHandler.startForeground()
        }

        // We always need to return a binder. If the system binds to our VPN service, VpnService
        // will return a binder that shall be user, otherwise we return an empty dummy binder to
        // keep connection service alive since the actual communication happens over gRPC.
        return super.onBind(intent) ?: emptyBinder()
    }

    override fun onRebind(intent: Intent?) {
        super.onRebind(intent)
        val count = bindCount.incrementAndGet()
        Logger.i("onRebind: $intent, bindCount: $count")

        if (intent.isFromSystem()) {
            Logger.i("onRebind from system")
            foregroundNotificationHandler.startForeground()
        }
    }

    private fun startDaemon(daemonConfig: DaemonConfig) =
        with(daemonConfig) {
            MullvadDaemon.initialize(
                vpnService = this@MullvadVpnService,
                rpcSocketPath = rpcSocket.absolutePath,
                filesDirectory = filesDir.absolutePath,
                cacheDirectory = cacheDir.absolutePath,
                apiEndpointOverride = apiEndpointOverride,
                extraMetadata = mapOf("flavor" to BuildConfig.FLAVOR),
            )
            Logger.i("MullvadVpnService: Daemon initialized")
        }

    private fun emptyBinder() =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            Binder(this.toString())
        } else {
            Binder()
        }

    override fun onRevoke() {
        Logger.d("onRevoke")
        runBlocking { connectionProxy.disconnect() }
    }

    override fun onUnbind(intent: Intent): Boolean {
        val count = bindCount.decrementAndGet()
        Logger.i("onUnbind: $intent, bindCount: $count")

        // Foreground?
        if (intent.isFromSystem()) {
            Logger.i("onUnbind from system")
            foregroundNotificationHandler.stopForeground()
        }

        return true
    }

    override fun onDestroy() {
        super.onDestroy()
        Logger.i("MullvadVpnService: onDestroy")
        // Shutting down the daemon gracefully
        managementService.stop()

        Logger.i("Shutdown MullvadDaemon")
        MullvadDaemon.shutdown()

        Logger.i("Enter Idle")
        managementService.enterIdle()

        Logger.i("Shutdown complete")
    }

    // If an intent is from the system it is because of the OS starting/stopping the VPN.
    private fun Intent?.isFromSystem(): Boolean {
        return this?.action == SERVICE_INTERFACE
    }

    private fun Context.prepareFiles() {
        extractAndOverwriteIfAssetMoreRecent(
            RELAY_LIST_ASSET_NAME,
            BuildConfig.REQUIRE_BUNDLED_RELAY_FILE,
        )
    }

    companion object {
        init {
            System.loadLibrary("mullvad_jni")
        }
    }
}

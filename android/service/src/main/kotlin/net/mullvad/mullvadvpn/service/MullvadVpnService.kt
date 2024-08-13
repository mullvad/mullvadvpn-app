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
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.lib.common.constant.BuildTypes
import net.mullvad.mullvadvpn.lib.common.constant.GRPC_SOCKET_FILE_NAMED_ARGUMENT
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
import net.mullvad.mullvadvpn.service.migration.MigrateSplitTunneling
import net.mullvad.mullvadvpn.service.notifications.ForegroundNotificationManager
import net.mullvad.mullvadvpn.service.notifications.NotificationChannelFactory
import net.mullvad.mullvadvpn.service.notifications.NotificationManager
import net.mullvad.talpid.TalpidVpnService
import org.koin.android.ext.android.getKoin
import org.koin.core.context.loadKoinModules
import org.koin.core.qualifier.named

private const val RELAYS_FILE = "relays.json"

class MullvadVpnService : TalpidVpnService() {

    private lateinit var keyguardManager: KeyguardManager

    private lateinit var apiEndpointConfiguration: ApiEndpointConfiguration
    private lateinit var managementService: ManagementService
    private lateinit var migrateSplitTunneling: MigrateSplitTunneling
    private lateinit var intentProvider: IntentProvider
    private lateinit var connectionProxy: ConnectionProxy
    private lateinit var rpcSocketFile: File

    private lateinit var foregroundNotificationHandler: ForegroundNotificationManager

    // Count number of binds to know if the service is needed. If user actively using the VPN, a
    // bind from the system, should always be present.
    private val bindCount = AtomicInt()

    override fun onCreate() {
        super.onCreate()
        Logger.i("MullvadVpnService: onCreate")

        loadKoinModules(listOf(vpnServiceModule, apiEndpointModule))
        with(getKoin()) {
            // Needed to create all the notification channels
            get<NotificationChannelFactory>()

            managementService = get()

            foregroundNotificationHandler =
                ForegroundNotificationManager(this@MullvadVpnService, get())
            get<NotificationManager>()

            apiEndpointConfiguration = get()
            migrateSplitTunneling = get()
            intentProvider = get()
            connectionProxy = get()
            rpcSocketFile = get(named(GRPC_SOCKET_FILE_NAMED_ARGUMENT))
        }

        keyguardManager = getSystemService<KeyguardManager>()!!

        // TODO We should avoid lifecycleScope.launch (current needed due to InetSocketAddress
        // with intent from API)
        lifecycleScope.launch(context = Dispatchers.IO) {
            prepareFiles(this@MullvadVpnService)
            migrateSplitTunneling.migrate()

            Logger.i("Start daemon")
            startDaemon()

            Logger.i("Start management service")
            managementService.start()
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
                // Only show on foreground if we have permission
                if (prepare(this) == null) {
                    foregroundNotificationHandler.startForeground()
                }
                lifecycleScope.launch { connectionProxy.connectWithoutPermissionCheck() }
            }
            intent?.action == KEY_DISCONNECT_ACTION -> {
                lifecycleScope.launch { connectionProxy.disconnect() }
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

    private fun startDaemon() {
        val apiEndpointConfiguration =
            if (BuildConfig.DEBUG) {
                intentProvider.getLatestIntent()?.getApiEndpointConfigurationExtras()
                    ?: apiEndpointConfiguration
            } else {
                apiEndpointConfiguration
            }

        MullvadDaemon.initialize(
            vpnService = this@MullvadVpnService,
            rpcSocketPath = rpcSocketFile.absolutePath,
            filesDirectory = filesDir.absolutePath,
            cacheDirectory = cacheDir.absolutePath,
            apiEndpoint = apiEndpointConfiguration.apiEndpoint()
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

        if (count == 0) {
            Logger.i("No one bound to the service, stopSelf()")
            lifecycleScope.launch {
                Logger.i("Waiting for disconnected state")
                // TODO This needs reworking, we should not wait for the disconnected state, what we
                // want is the notification of disconnected to go out before we start shutting down
                connectionProxy.tunnelState
                    .filter {
                        it is TunnelState.Disconnected ||
                            (it is TunnelState.Error && !it.errorState.isBlocking)
                    }
                    .first()

                if (bindCount.get() == 0) {
                    Logger.i("Stopping service")
                    stopSelf()
                }
            }
        }
        return true
    }

    override fun onDestroy() {
        Logger.i("MullvadVpnService: onDestroy")
        // Shutting down the daemon gracefully
        managementService.stop()

        Logger.i("Shutdown MullvadDaemon")
        MullvadDaemon.shutdown()
        Logger.i("Enter Idle")
        managementService.enterIdle()
        Logger.i("Shutdown complete")
        super.onDestroy()
    }

    // If an intent is from the system it is because of the OS starting/stopping the VPN.
    private fun Intent?.isFromSystem(): Boolean {
        return this?.action == SERVICE_INTERFACE
    }

    private fun prepareFiles(context: Context) {
        val shouldOverwriteRelayList =
            lastUpdatedTime(context) > File(context.filesDir, RELAYS_FILE).lastModified()

        FileResourceExtractor(context).apply { extract(RELAYS_FILE, shouldOverwriteRelayList) }
    }

    private fun lastUpdatedTime(context: Context): Long =
        context.packageManager.getPackageInfo(context.packageName, 0).lastUpdateTime

    companion object {
        init {
            System.loadLibrary("mullvad_jni")
        }
    }
}

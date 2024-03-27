package net.mullvad.mullvadvpn.service

import android.annotation.SuppressLint
import android.content.Context
import java.io.File
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpoint
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration

private const val RELAYS_FILE = "relays.json"

@SuppressLint("SdCardPath")
class MullvadDaemon(
    vpnService: MullvadVpnService,
    apiEndpointConfiguration: ApiEndpointConfiguration
) {
    protected var daemonInterfaceAddress = 0L

    var onDaemonStopped: (() -> Unit)? = null

    init {
        // This should be done by the daemon
        File("/data/data/net.mullvad.mullvadvpn/rpc-socket").delete()

        System.loadLibrary("mullvad_jni")

        prepareFiles(vpnService)

        initialize(
            vpnService = vpnService,
            cacheDirectory = vpnService.cacheDir.absolutePath,
            resourceDirectory = vpnService.filesDir.absolutePath,
            apiEndpoint = apiEndpointConfiguration.apiEndpoint()
        )
    }

    fun onDestroy() {
        onDaemonStopped = null
        shutdown()
        deinitialize()
    }

    private external fun initialize(
        vpnService: MullvadVpnService,
        cacheDirectory: String,
        resourceDirectory: String,
        apiEndpoint: ApiEndpoint?
    )

    private external fun deinitialize()

    private external fun shutdown()

    // Used by JNI
    @Suppress("unused")
    private fun notifyDaemonStopped() {
        onDaemonStopped?.invoke()
    }

    private fun prepareFiles(context: Context) {
        val shouldOverwriteRelayList =
            lastUpdatedTime(context) > File(context.filesDir, RELAYS_FILE).lastModified()

        FileResourceExtractor(context).apply { extract(RELAYS_FILE, shouldOverwriteRelayList) }
    }

    private fun lastUpdatedTime(context: Context): Long =
        context.packageManager.getPackageInfo(context.packageName, 0).lastUpdateTime
}

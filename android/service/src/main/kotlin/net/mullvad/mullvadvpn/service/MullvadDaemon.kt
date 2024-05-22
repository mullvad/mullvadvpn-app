package net.mullvad.mullvadvpn.service

import android.annotation.SuppressLint
import android.content.Context
import java.io.File
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpoint
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.repository.MigrateSplitTunnelingRepository

private const val RELAYS_FILE = "relays.json"

@SuppressLint("SdCardPath")
class MullvadDaemon(
    vpnService: MullvadVpnService,
    apiEndpointConfiguration: ApiEndpointConfiguration,
    migrateSplitTunnelingRepository: MigrateSplitTunnelingRepository
) {
    protected var daemonInterfaceAddress = 0L

    var onDaemonStopped: (() -> Unit)? = null

    init {
        System.loadLibrary("mullvad_jni")

        prepareFiles(vpnService)

        migrateSplitTunnelingRepository.migrateSplitTunneling()

        initialize(
            vpnService = vpnService,
            cacheDirectory = vpnService.cacheDir.absolutePath,
            resourceDirectory = vpnService.filesDir.absolutePath,
            apiEndpoint = apiEndpointConfiguration.apiEndpoint()
        )
    }

    fun onDestroy() {
        onDaemonStopped = null
        // TODO: Stop ManagementService
        shutdown(daemonInterfaceAddress)
        deinitialize()
    }

    private external fun initialize(
        vpnService: MullvadVpnService,
        cacheDirectory: String,
        resourceDirectory: String,
        apiEndpoint: ApiEndpoint?
    )

    external fun deinitialize()

    external fun shutdown(daemonInterfaceAddress: Long)

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

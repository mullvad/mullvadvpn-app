package net.mullvad.mullvadvpn.service

import android.annotation.SuppressLint
import android.content.Context
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpoint
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.service.migration.MigrateSplitTunneling

private const val RELAYS_FILE = "relays.json"

@SuppressLint("SdCardPath")
class MullvadDaemon(
    vpnService: MullvadVpnService,
    rpcSocketFile: File,
    apiEndpointConfiguration: ApiEndpointConfiguration,
    migrateSplitTunneling: MigrateSplitTunneling
) {
    init {
        System.loadLibrary("mullvad_jni")

        prepareFiles(vpnService)
        migrateSplitTunneling.migrate()

        init(
            vpnService = vpnService,
            rpcSocketPath = rpcSocketFile.absolutePath,
            filesDirectory = vpnService.filesDir.absolutePath,
            cacheDirectory = vpnService.cacheDir.absolutePath,
            apiEndpoint = apiEndpointConfiguration.apiEndpoint()
        )
    }

    private fun prepareFiles(context: Context) {
        val shouldOverwriteRelayList =
            lastUpdatedTime(context) > File(context.filesDir, RELAYS_FILE).lastModified()

        FileResourceExtractor(context).apply { extract(RELAYS_FILE, shouldOverwriteRelayList) }
    }

    private fun lastUpdatedTime(context: Context): Long =
        context.packageManager.getPackageInfo(context.packageName, 0).lastUpdateTime

    private external fun init(
        vpnService: MullvadVpnService,
        rpcSocketPath: String,
        filesDirectory: String,
        cacheDirectory: String,
        apiEndpoint: ApiEndpoint?
    )

    public external fun shutdown()
}

package net.mullvad.mullvadvpn.service

import android.annotation.SuppressLint
import java.io.File
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpoint
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration

@SuppressLint("SdCardPath")
class MullvadDaemon {
    companion object {
        init {
            System.loadLibrary("mullvad_jni")
        }

        public fun start(
            vpnService: MullvadVpnService,
            rpcSocketFile: File,
            apiEndpointConfiguration: ApiEndpointConfiguration,
        ) {
            initialize(
                vpnService = vpnService,
                rpcSocketPath = rpcSocketFile.absolutePath,
                filesDirectory = vpnService.filesDir.absolutePath,
                cacheDirectory = vpnService.cacheDir.absolutePath,
                apiEndpoint = apiEndpointConfiguration.apiEndpoint()
            )
        }

        private external fun initialize(
            vpnService: MullvadVpnService,
            rpcSocketPath: String,
            filesDirectory: String,
            cacheDirectory: String,
            apiEndpoint: ApiEndpoint?
        )

        public external fun shutdown()
    }
}

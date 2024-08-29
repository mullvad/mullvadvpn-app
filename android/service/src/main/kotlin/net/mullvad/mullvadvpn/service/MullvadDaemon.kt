package net.mullvad.mullvadvpn.service

import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpoint

object MullvadDaemon {
    init {
        System.loadLibrary("mullvad_jni")
    }

    external fun initialize(
        vpnService: MullvadVpnService,
        rpcSocketPath: String,
        filesDirectory: String,
        cacheDirectory: String,
        apiEndpoint: ApiEndpoint?,
    )

    external fun shutdown()
}

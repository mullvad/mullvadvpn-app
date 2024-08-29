package net.mullvad.mullvadvpn.service

import java.io.File
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointOverride

data class DaemonConfig(
    val rpcSocket: File,
    val filesDir: File,
    val cacheDir: File,
    val apiEndpointOverride: ApiEndpointOverride?
)

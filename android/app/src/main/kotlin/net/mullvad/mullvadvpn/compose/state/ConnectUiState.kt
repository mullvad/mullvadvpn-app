package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.talpid.net.TransportProtocol

data class ConnectUiState(
    val location: GeoIpLocation?,
    val relayLocation: RelayItem?,
    val versionInfo: VersionInfo?,
    val tunnelUiState: TunnelState,
    val tunnelRealState: TunnelState,
    val inAddress: Triple<String, Int, TransportProtocol>?,
    val outAddress: String,
    val showLocation: Boolean,
    val isTunnelInfoExpanded: Boolean
) {
    companion object {
        val INITIAL =
            ConnectUiState(
                location = null,
                relayLocation = null,
                versionInfo = null,
                tunnelUiState = TunnelState.Disconnected,
                tunnelRealState = TunnelState.Disconnected,
                inAddress = null,
                outAddress = "",
                showLocation = false,
                isTunnelInfoExpanded = false
            )
    }
}

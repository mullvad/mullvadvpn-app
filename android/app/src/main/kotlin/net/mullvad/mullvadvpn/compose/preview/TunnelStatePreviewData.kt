package net.mullvad.mullvadvpn.compose.preview

import java.net.InetSocketAddress
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.Endpoint
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.ObfuscationEndpoint
import net.mullvad.mullvadvpn.lib.model.ObfuscationType
import net.mullvad.mullvadvpn.lib.model.TransportProtocol
import net.mullvad.mullvadvpn.lib.model.TunnelEndpoint
import net.mullvad.mullvadvpn.lib.model.TunnelState

object TunnelStatePreviewData {
    fun generateDisconnectedState() = TunnelState.Disconnected()

    fun generateConnectingState(featureIndicators: Int, quantumResistant: Boolean) =
        TunnelState.Connecting(
            endpoint = generateTunnelEndpoint(quantumResistant = quantumResistant, daita = false),
            location = generateLocation(),
            featureIndicators = generateFeatureIndicators(featureIndicators),
        )

    fun generateConnectedState(featureIndicators: Int, quantumResistant: Boolean) =
        TunnelState.Connected(
            endpoint = generateTunnelEndpoint(quantumResistant = quantumResistant, daita = true),
            location = generateLocation(),
            featureIndicators = generateFeatureIndicators(featureIndicators),
        )

    fun generateDisconnectingState(actionAfterDisconnect: ActionAfterDisconnect) =
        TunnelState.Disconnecting(actionAfterDisconnect = actionAfterDisconnect)

    fun generateErrorState(isBlocking: Boolean) =
        TunnelState.Error(
            errorState = ErrorState(cause = ErrorStateCause.DnsError, isBlocking = isBlocking)
        )

    private fun generateTunnelEndpoint(quantumResistant: Boolean, daita: Boolean): TunnelEndpoint =
        TunnelEndpoint(
            entryEndpoint = null,
            endpoint = generateEndpoint(TransportProtocol.Udp),
            quantumResistant = quantumResistant,
            obfuscation =
                ObfuscationEndpoint(
                    endpoint = generateEndpoint(TransportProtocol.Tcp),
                    ObfuscationType.Udp2Tcp,
                ),
            daita = daita,
        )

    private fun generateEndpoint(transportProtocol: TransportProtocol) =
        Endpoint(address = InetSocketAddress(DEFAULT_ENDPOINT_PORT), protocol = transportProtocol)

    private fun generateLocation(): GeoIpLocation =
        GeoIpLocation(
            ipv4 = null,
            ipv6 = null,
            country = "",
            city = "",
            hostname = "",
            entryHostname = "",
            latitude = 0.0,
            longitude = 0.0,
        )

    private fun generateFeatureIndicators(size: Int): List<FeatureIndicator> =
        FeatureIndicator.entries.subList(0, size)
}

private const val DEFAULT_ENDPOINT_PORT = 100

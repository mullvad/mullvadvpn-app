package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.AlphaInactive
import net.mullvad.mullvadvpn.compose.theme.AlphaInvisible
import net.mullvad.mullvadvpn.compose.theme.AlphaVisible
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.talpid.net.TransportProtocol
import net.mullvad.talpid.net.TunnelEndpoint

@Preview
@Composable
fun PreviewLocationInfo() {
    AppTheme {
        LocationInfo(
            onToggleTunnelInfo = {},
            visible = true,
            expanded = true,
            location = null,
            tunnelEndpoint = null
        )
    }
}

@Composable
fun LocationInfo(
    modifier: Modifier = Modifier,
    colorExpanded: Color = MaterialTheme.colorScheme.onPrimary,
    colorCollapsed: Color = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaInactive),
    onToggleTunnelInfo: () -> Unit,
    visible: Boolean,
    expanded: Boolean,
    location: GeoIpLocation?,
    tunnelEndpoint: TunnelEndpoint?
) {
    Column(
        modifier =
            if (visible) {
                    Modifier.clickable { onToggleTunnelInfo() }.alpha(AlphaVisible)
                } else {
                    Modifier.alpha(AlphaInvisible)
                }
                .then(modifier)
    ) {
        Row {
            Text(
                text = location?.hostname ?: "",
                color =
                    if (expanded) {
                        colorExpanded
                    } else {
                        colorCollapsed
                    },
                style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.SemiBold)
            )
            ChevronView(
                isExpanded = expanded,
                colorFilter =
                    ColorFilter.tint(
                        if (expanded) {
                            colorExpanded
                        } else {
                            colorCollapsed
                        }
                    ),
                modifier = Modifier.padding(horizontal = Dimens.chevronMargin)
            )
        }
        Text(
            text =
                if (expanded) {
                    stringResource(id = R.string.wireguard)
                } else {
                    ""
                },
            color = colorExpanded,
            style = MaterialTheme.typography.labelMedium
        )
        // In address
        val inAddress =
            if (expanded && tunnelEndpoint != null) {
                val relayEndpoint = tunnelEndpoint.obfuscation?.endpoint ?: tunnelEndpoint.endpoint
                val host = relayEndpoint.address.address.hostAddress ?: ""
                val port = relayEndpoint.address.port
                val protocol =
                    when (relayEndpoint.protocol) {
                        TransportProtocol.Tcp -> stringResource(id = R.string.tcp)
                        TransportProtocol.Udp -> stringResource(id = R.string.udp)
                    }
                "${stringResource(id = R.string.in_address)} $host:$port $protocol"
            } else {
                ""
            }
        Text(text = inAddress, color = colorExpanded, style = MaterialTheme.typography.labelMedium)
        // Out address
        val outAddress =
            if (expanded && location != null && (location.ipv4 != null || location.ipv6 != null)) {
                val ipv4 = location.ipv4
                val ipv6 = location.ipv6

                if (ipv6 == null) {
                    "${stringResource(id = R.string.out_address)} ${ipv4?.hostAddress ?: ""}"
                } else if (ipv4 == null) {
                    "${stringResource(id = R.string.out_address)} ${ipv6.hostAddress ?: ""}"
                } else {
                    "${stringResource(id = R.string.out_address)} ${ipv4.hostAddress} / ${ipv6.hostAddress}"
                }
            } else {
                ""
            }
        Text(text = outAddress, color = colorExpanded, style = MaterialTheme.typography.labelMedium)
    }
}

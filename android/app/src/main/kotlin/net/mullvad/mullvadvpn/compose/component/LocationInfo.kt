package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.test.LOCATION_INFO_CONNECTION_OUT_TEST_TAG
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.TransportProtocol
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInvisible
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible

@Preview
@Composable
private fun PreviewLocationInfo() {
    AppTheme {
        LocationInfo(
            onToggleTunnelInfo = {},
            isVisible = true,
            isExpanded = true,
            location = null,
            isUsingDaita = false,
            inAddress = null,
            outAddress = "",
        )
    }
}

@Composable
fun LocationInfo(
    modifier: Modifier = Modifier,
    colorExpanded: Color = MaterialTheme.colorScheme.onSurface,
    colorCollapsed: Color = MaterialTheme.colorScheme.onSurfaceVariant,
    onToggleTunnelInfo: () -> Unit,
    isVisible: Boolean,
    isExpanded: Boolean,
    location: GeoIpLocation?,
    isUsingDaita: Boolean,
    inAddress: Triple<String, Int, TransportProtocol>?,
    outAddress: String,
) {
    Column(
        modifier =
            if (isVisible) {
                    Modifier.clickable { onToggleTunnelInfo() }.alpha(AlphaVisible)
                } else {
                    Modifier.alpha(AlphaInvisible)
                }
                .then(modifier)
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            RelayHostname(
                hostname = location?.hostname,
                isUsingDaita = isUsingDaita,
                isExpanded = isExpanded,
                colorExpanded = colorExpanded,
                colorCollapsed = colorCollapsed,
            )
            Chevron(
                isExpanded = isExpanded,
                color =
                    if (isExpanded) {
                        colorExpanded
                    } else {
                        colorCollapsed
                    },
                modifier = Modifier.padding(horizontal = Dimens.chevronMargin),
            )
        }
        Text(
            text =
                if (isExpanded) {
                    stringResource(id = R.string.wireguard)
                } else {
                    ""
                },
            color = colorExpanded,
            style = MaterialTheme.typography.labelMedium,
        )
        val textInAddress =
            inAddress?.let {
                val protocol =
                    when (inAddress.third) {
                        TransportProtocol.Tcp -> stringResource(id = R.string.tcp)
                        TransportProtocol.Udp -> stringResource(id = R.string.udp)
                    }
                "${inAddress.first}:${inAddress.second} $protocol"
            } ?: ""
        Text(
            text = "${stringResource(id = R.string.in_address)} $textInAddress",
            color = colorExpanded,
            style = MaterialTheme.typography.labelMedium,
            modifier = Modifier.alpha(if (isExpanded) AlphaVisible else AlphaInvisible),
        )
        Text(
            text = "${stringResource(id = R.string.out_address)} $outAddress",
            color = colorExpanded,
            style = MaterialTheme.typography.labelMedium,
            modifier =
                Modifier.testTag(LOCATION_INFO_CONNECTION_OUT_TEST_TAG)
                    .alpha(
                        if (isExpanded && outAddress.isNotEmpty()) AlphaVisible else AlphaInvisible
                    ),
        )
    }
}

@Composable
private fun RelayHostname(
    hostname: String?,
    isUsingDaita: Boolean,
    isExpanded: Boolean,
    colorExpanded: Color,
    colorCollapsed: Color,
) {
    val hostnameTitle =
        when {
            hostname != null && isUsingDaita -> {
                stringResource(
                    id = R.string.connected_using_daita,
                    hostname,
                    stringResource(id = R.string.daita),
                )
            }
            hostname != null -> hostname
            else -> ""
        }

    Text(
        text = hostnameTitle,
        color =
            if (isExpanded) {
                colorExpanded
            } else {
                colorCollapsed
            },
        style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.SemiBold),
    )
}

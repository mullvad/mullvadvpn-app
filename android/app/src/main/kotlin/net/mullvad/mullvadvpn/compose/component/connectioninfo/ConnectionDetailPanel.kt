package net.mullvad.mullvadvpn.compose.component.connectioninfo

import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.ConnectionDetails
import net.mullvad.mullvadvpn.compose.test.LOCATION_INFO_CONNECTION_IN_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LOCATION_INFO_CONNECTION_OUT_TEST_TAG
import net.mullvad.mullvadvpn.constant.SPACE_CHAR
import net.mullvad.mullvadvpn.lib.model.TransportProtocol
import net.mullvad.mullvadvpn.lib.model.TunnelEndpoint
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Composable
fun ConnectionDetailPanel(connectionDetails: ConnectionDetails) {

    ConnectionInfoHeader(
        stringResource(R.string.connect_panel_connection_details),
        Modifier.fillMaxWidth().padding(bottom = Dimens.smallPadding),
    )

    AnimatedContent(connectionDetails, label = "ConnectionDetails") {
        ConnectionDetails(
            it.inAddress,
            it.outIpv4Address,
            it.outIpv6Address,
            modifier = Modifier.padding(bottom = Dimens.smallPadding),
        )
    }
}

@Suppress("LongMethod")
@Composable
fun ConnectionDetails(
    inIPV4: String,
    outIPV4: String?,
    outIPV6: String?,
    modifier: Modifier = Modifier,
) {
    ConstraintLayout(modifier = modifier.fillMaxWidth()) {
        val (inAddrHeader, inAddr, outAddrV4Header, outAddrV4, outAddrV6Header, outAddrV6) =
            createRefs()
        val headerBarrier = createEndBarrier(inAddrHeader, outAddrV4Header, outAddrV6Header)

        val inAddrBarrier = createBottomBarrier(inAddrHeader, inAddr)
        val outAddrV4Barrier = createBottomBarrier(inAddrHeader, inAddr, outAddrV4Header, outAddrV4)

        val outAddrV6Barrier =
            createBottomBarrier(
                inAddrHeader,
                inAddr,
                outAddrV4Header,
                outAddrV4,
                outAddrV6Header,
                outAddrV6,
            )

        Text(
            text = stringResource(R.string.connection_details_in),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.bodySmall,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier =
                Modifier.padding(end = Dimens.smallPadding).constrainAs(inAddrHeader) {
                    start.linkTo(parent.start)
                    top.linkTo(parent.top)
                    bottom.linkTo(inAddrBarrier)
                    height = Dimension.wrapContent
                    width = Dimension.wrapContent
                },
        )
        Text(
            text = inIPV4,
            color = MaterialTheme.colorScheme.onPrimary,
            style = MaterialTheme.typography.bodySmall,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier =
                Modifier.testTag(LOCATION_INFO_CONNECTION_IN_TEST_TAG).constrainAs(inAddr) {
                    start.linkTo(headerBarrier)
                    end.linkTo(parent.end)
                    top.linkTo(parent.top)
                    bottom.linkTo(inAddrBarrier)
                    height = Dimension.wrapContent
                    width = Dimension.fillToConstraints
                },
        )

        if (outIPV4 != null) {
            Text(
                text =
                    buildString {
                        append(stringResource(R.string.connection_details_out))
                        append(SPACE_CHAR)
                        append(stringResource(R.string.ipv4))
                    },
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier =
                    Modifier.padding(end = Dimens.smallPadding).constrainAs(outAddrV4Header) {
                        start.linkTo(parent.start)
                        top.linkTo(inAddrBarrier)
                        bottom.linkTo(outAddrV4Barrier)
                        height = Dimension.wrapContent
                        width = Dimension.wrapContent
                    },
            )
            Box(
                modifier =
                    Modifier.constrainAs(outAddrV4) {
                        start.linkTo(headerBarrier)
                        end.linkTo(parent.end)
                        top.linkTo(inAddrBarrier)
                        bottom.linkTo(outAddrV4Barrier)
                        height = Dimension.wrapContent
                        width = Dimension.fillToConstraints
                    }
            ) {
                SelectionContainer {
                    Text(
                        modifier = Modifier.testTag(LOCATION_INFO_CONNECTION_OUT_TEST_TAG),
                        text = outIPV4,
                        color = MaterialTheme.colorScheme.onPrimary,
                        style = MaterialTheme.typography.bodySmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }

        if (outIPV6 != null) {
            Text(
                text =
                    buildString {
                        append(stringResource(R.string.connection_details_out))
                        append(SPACE_CHAR)
                        append(stringResource(R.string.ipv6))
                    },
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier =
                    Modifier.padding(end = Dimens.smallPadding).constrainAs(outAddrV6Header) {
                        start.linkTo(parent.start)
                        top.linkTo(outAddrV4Barrier)
                        bottom.linkTo(outAddrV6Barrier)
                        height = Dimension.wrapContent
                        width = Dimension.wrapContent
                    },
            )
            Box(
                modifier =
                    Modifier.constrainAs(outAddrV6) {
                        start.linkTo(headerBarrier)
                        end.linkTo(parent.end)
                        top.linkTo(outAddrV4Barrier)
                        bottom.linkTo(outAddrV6Barrier)
                        height = Dimension.wrapContent
                        width = Dimension.fillToConstraints
                    }
            ) {
                SelectionContainer {
                    Text(
                        text = outIPV6,
                        color = MaterialTheme.colorScheme.onPrimary,
                        style = MaterialTheme.typography.bodySmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }
    }
}

@Composable
fun TunnelEndpoint.toInAddress(): String {
    val relayEndpoint = this.obfuscation?.endpoint ?: this.endpoint

    val host = relayEndpoint.address.address.hostAddress ?: ""
    val port = relayEndpoint.address.port
    val protocol = relayEndpoint.protocol

    return buildString {
        append(host)
        append(":")
        append(port)
        append(" ")
        append(
            when (protocol) {
                TransportProtocol.Tcp -> stringResource(id = R.string.tcp)
                TransportProtocol.Udp -> stringResource(id = R.string.udp)
            }
        )
    }
}

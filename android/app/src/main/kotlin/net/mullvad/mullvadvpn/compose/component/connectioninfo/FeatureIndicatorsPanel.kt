package net.mullvad.mullvadvpn.compose.component.connectioninfo

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ContextualFlowRow
import androidx.compose.foundation.layout.ContextualFlowRowOverflow
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.MullvadFeatureChip
import net.mullvad.mullvadvpn.compose.component.MullvadMoreChip
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Composable
fun FeatureIndicatorsPanel(
    featureIndicators: List<FeatureIndicator>,
    expanded: Boolean,
    onToggleExpand: () -> Unit,
) {
    if (featureIndicators.isNotEmpty()) {
        if (expanded) {
            ConnectionInfoHeader(
                stringResource(R.string.connect_panel_active_features),
                Modifier.fillMaxWidth(),
            )
        }
        FeatureIndicators(featureIndicators, expanded, onToggleExpand)
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun FeatureIndicators(
    features: List<FeatureIndicator>,
    expanded: Boolean,
    onToggleExpand: () -> Unit,
) {
    ContextualFlowRow(
        modifier = Modifier.fillMaxWidth(),
        itemCount = features.size,
        // FlowRow may crash if maxLines is set to 1
        // https://issuetracker.google.com/issues/367440149 &
        // https://issuetracker.google.com/issues/355003185
        maxLines = if (expanded) Int.MAX_VALUE else 2,
        horizontalArrangement = Arrangement.spacedBy(Dimens.smallPadding),
        overflow =
            ContextualFlowRowOverflow.expandOrCollapseIndicator(
                expandIndicator = {
                    val hiddenFeatureCount = totalItemCount - shownItemCount
                    MullvadMoreChip(
                        onClick = onToggleExpand,
                        text =
                            stringResource(
                                R.string.feature_indicators_show_more,
                                hiddenFeatureCount,
                            ),
                        containerColor = Color.Transparent,
                    )
                },
                collapseIndicator = {},
            ),
    ) { index ->
        MullvadFeatureChip(text = features[index].text())
    }

    // Spacing are added to compensate for when there are no feature indicators, since each feature
    // indicator has built-in padding. Padding looks the same towards Switch Location button with or
    // without feature indicators.
    if (features.isEmpty() && !expanded) {
        Spacer(Modifier.height(Dimens.smallSpacer))
    }
}

@Composable
private fun FeatureIndicator.text(): String {
    val resource =
        when (this) {
            FeatureIndicator.QUANTUM_RESISTANCE -> R.string.feature_quantum_resistant
            FeatureIndicator.SPLIT_TUNNELING -> R.string.feature_split_tunneling
            FeatureIndicator.SHADOWSOCKS,
            FeatureIndicator.UDP_2_TCP -> R.string.feature_udp_2_tcp
            FeatureIndicator.LAN_SHARING -> R.string.local_network_sharing
            FeatureIndicator.DNS_CONTENT_BLOCKERS -> R.string.dns_content_blockers
            FeatureIndicator.CUSTOM_DNS -> R.string.feature_custom_dns
            FeatureIndicator.SERVER_IP_OVERRIDE -> R.string.feature_server_ip_override
            FeatureIndicator.CUSTOM_MTU -> R.string.feature_custom_mtu
            FeatureIndicator.DAITA -> R.string.daita
            FeatureIndicator.MULTIHOP -> R.string.feature_multihop
        }
    return textResource(resource)
}

package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorSmall
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.theme.color.selected

@Preview
@Composable
private fun PreviewServerIpOverridesCell() {
    AppTheme { ServerIpOverridesCell(active = true) }
}

@Composable
fun ServerIpOverridesCell(
    active: Boolean?,
    modifier: Modifier = Modifier,
    activeColor: Color = MaterialTheme.colorScheme.selected,
    inactiveColor: Color = MaterialTheme.colorScheme.error,
) {
    BaseCell(
        modifier = modifier,
        iconView = {
            if (active == null) {
                MullvadCircularProgressIndicatorSmall()
            } else {
                Box(
                    modifier =
                        Modifier.size(Dimens.relayCircleSize)
                            .background(
                                color =
                                    when {
                                        active -> activeColor
                                        else -> inactiveColor
                                    },
                                shape = CircleShape
                            )
                )
            }
        },
        headlineContent = {
            if (active != null) {
                Text(
                    text =
                        if (active) stringResource(id = R.string.server_ip_overrides_active)
                        else stringResource(id = R.string.server_ip_overrides_inactive),
                    color = MaterialTheme.colorScheme.onPrimary,
                    modifier =
                        Modifier.weight(1f)
                            .alpha(
                                if (active) {
                                    AlphaVisible
                                } else {
                                    AlphaInactive
                                }
                            )
                            .padding(
                                horizontal = Dimens.smallPadding,
                                vertical = Dimens.mediumPadding
                            )
                )
            }
        },
        isRowEnabled = false
    )
}

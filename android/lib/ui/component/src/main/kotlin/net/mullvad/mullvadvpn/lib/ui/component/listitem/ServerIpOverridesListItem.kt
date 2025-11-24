package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.selected
import net.mullvad.mullvadvpn.lib.ui.component.R
import net.mullvad.mullvadvpn.lib.ui.component.preview.PreviewSpacedColumn
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorSmall
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem

@Preview
@Composable
private fun PreviewServerIpOverridesListItem() {
    PreviewSpacedColumn {
        AppTheme { ServerIpOverridesListItem(active = true) }
        AppTheme { ServerIpOverridesListItem(active = false) }
        AppTheme { ServerIpOverridesListItem(active = null) }
    }
}

@Composable
fun ServerIpOverridesListItem(
    active: Boolean?,
    modifier: Modifier = Modifier,
    activeColor: Color = MaterialTheme.colorScheme.selected,
    inactiveColor: Color = MaterialTheme.colorScheme.error,
) {
    MullvadListItem(
        modifier = modifier,
        isEnabled = active == true,
        leadingContent = {
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
                                shape = CircleShape,
                            )
                )
            }
        },
        content = {
            if (active != null) {
                Text(
                    text =
                        if (active) stringResource(id = R.string.server_ip_overrides_active)
                        else stringResource(id = R.string.server_ip_overrides_inactive),
                    modifier = Modifier.padding(horizontal = Dimens.smallPadding),
                )
            }
        },
    )
}

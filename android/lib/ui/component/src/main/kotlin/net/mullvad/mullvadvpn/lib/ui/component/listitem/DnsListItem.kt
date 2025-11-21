package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Error
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.R
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Preview
@Composable
private fun PreviewDnsListItem() {
    AppTheme {
        DnsListItem(
            address = "0.0.0.0",
            isUnreachableLocalDnsWarningVisible = true,
            isUnreachableIpv6DnsWarningVisible = false,
            onClick = {},
        )
    }
}

@Composable
fun DnsListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    address: String,
    isUnreachableLocalDnsWarningVisible: Boolean,
    isUnreachableIpv6DnsWarningVisible: Boolean,
    onClick: () -> Unit,
) {
    MullvadListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        onClick = onClick,
        content = {
            Row {
                if (isUnreachableLocalDnsWarningVisible || isUnreachableIpv6DnsWarningVisible) {
                    Icon(
                        modifier = Modifier.padding(end = Dimens.smallPadding),
                        imageVector = Icons.Rounded.Error,
                        contentDescription =
                            if (isUnreachableLocalDnsWarningVisible) {
                                stringResource(id = R.string.confirm_local_dns)
                            } else {
                                stringResource(id = R.string.confirm_ipv6_dns)
                            },
                        tint = MaterialTheme.colorScheme.error,
                    )
                }

                Text(address)
            }
        },
    )
}

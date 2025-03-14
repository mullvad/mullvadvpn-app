package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.RowScope
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.rounded.Error
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewDnsCell() {
    AppTheme {
        DnsCell(
            address = "0.0.0.0",
            isUnreachableLocalDnsWarningVisible = true,
            isUnreachableIpv6DnsWarningVisible = false,
            onClick = {},
        )
    }
}

@Composable
fun DnsCell(
    address: String,
    isUnreachableLocalDnsWarningVisible: Boolean,
    isUnreachableIpv6DnsWarningVisible: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val titleModifier = Modifier
    val startPadding = 54.dp

    BaseCell(
        headlineContent = { DnsTitle(address = address, modifier = titleModifier) },
        bodyView = {
            if (isUnreachableLocalDnsWarningVisible) {
                Icon(
                    imageVector = Icons.Rounded.Error,
                    contentDescription = stringResource(id = R.string.confirm_local_dns),
                    tint = MaterialTheme.colorScheme.error,
                )
            }
            if (isUnreachableIpv6DnsWarningVisible) {
                Icon(
                    imageVector = Icons.Rounded.Error,
                    contentDescription = stringResource(id = R.string.confirm_ipv6_dns),
                    tint = MaterialTheme.colorScheme.error,
                )
            }
        },
        onCellClicked = { onClick.invoke() },
        background = MaterialTheme.colorScheme.surfaceContainerLow,
        startPadding = startPadding,
        modifier = modifier,
    )
}

@Composable
private fun RowScope.DnsTitle(address: String, modifier: Modifier = Modifier) {
    Text(
        text = address,
        color = MaterialTheme.colorScheme.onSurface,
        style = MaterialTheme.typography.labelLarge,
        textAlign = TextAlign.Start,
        modifier = modifier.weight(1f),
    )
}

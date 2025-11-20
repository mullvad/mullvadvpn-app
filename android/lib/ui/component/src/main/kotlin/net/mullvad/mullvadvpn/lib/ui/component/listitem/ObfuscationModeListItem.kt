package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.layout.Column
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.component.DividerButton
import net.mullvad.mullvadvpn.lib.ui.component.R
import net.mullvad.mullvadvpn.lib.ui.component.preview.SelectObfuscationListItemPreviewParameterProvider
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Preview
@Composable
private fun PreviewObfuscationListItem(
    @PreviewParameter(SelectObfuscationListItemPreviewParameterProvider::class)
    selectedObfuscationCellData: Triple<ObfuscationMode, Constraint<Port>, Boolean>
) {
    AppTheme {
        ObfuscationModeListItem(
            hierarchy = Hierarchy.Child1,
            obfuscationMode = selectedObfuscationCellData.first,
            port = selectedObfuscationCellData.second,
            isSelected = selectedObfuscationCellData.third,
            onSelected = {},
            onNavigate = {},
        )
    }
}

@Composable
fun ObfuscationModeListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    obfuscationMode: ObfuscationMode,
    port: Constraint<Port>,
    isSelected: Boolean,
    onSelected: (ObfuscationMode) -> Unit,
    onNavigate: () -> Unit,
    testTag: String? = null,
) {
    SelectableListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        isSelected = isSelected,
        testTag = testTag,
        onClick = { onSelected(obfuscationMode) },
        content = {
            Column {
                Text(obfuscationMode.toTitle())
                Text(
                    stringResource(id = R.string.port_x, port.toSubTitle()),
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        },
        trailingContent = {
            DividerButton(onClick = onNavigate, icon = Icons.AutoMirrored.Filled.KeyboardArrowRight)
        },
    )
}

@Composable
private fun ObfuscationMode.toTitle() =
    when (this) {
        ObfuscationMode.Auto -> stringResource(id = R.string.automatic)
        ObfuscationMode.Off -> stringResource(id = R.string.off)
        ObfuscationMode.Udp2Tcp -> stringResource(id = R.string.udp_over_tcp)
        ObfuscationMode.Shadowsocks -> stringResource(id = R.string.shadowsocks)
        ObfuscationMode.Quic -> stringResource(id = R.string.quic)
        ObfuscationMode.Lwo -> stringResource(id = R.string.lwo)
    }

@Composable
private fun Constraint<Port>.toSubTitle() =
    when (this) {
        Constraint.Any -> stringResource(id = R.string.automatic)
        is Constraint.Only -> this.value.toString()
    }

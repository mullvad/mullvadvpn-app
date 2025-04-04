package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.preview.SelectObfuscationCellPreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.selected
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemSubText
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemText

@Preview
@Composable
private fun PreviewObfuscationCell(
    @PreviewParameter(SelectObfuscationCellPreviewParameterProvider::class)
    selectedObfuscationCellData: Triple<ObfuscationMode, Constraint<Port>, Boolean>
) {
    AppTheme {
        ObfuscationModeCell(
            obfuscationMode = selectedObfuscationCellData.first,
            port = selectedObfuscationCellData.second,
            isSelected = selectedObfuscationCellData.third,
            onSelected = {},
            onNavigate = {},
        )
    }
}

@Composable
fun ObfuscationModeCell(
    obfuscationMode: ObfuscationMode,
    port: Constraint<Port>,
    isSelected: Boolean,
    onSelected: (ObfuscationMode) -> Unit,
    onNavigate: () -> Unit,
    testTag: String? = null,
) {
    Row(
        modifier =
            Modifier.height(IntrinsicSize.Min)
                .fillMaxWidth()
                .background(MaterialTheme.colorScheme.surfaceContainerLow)
                .let { if (testTag != null) it.testTag(testTag) else it }
                .semantics { selected = isSelected }
    ) {
        TwoRowCell(
            modifier = Modifier.weight(1f),
            titleStyle = MaterialTheme.typography.listItemText,
            titleColor = MaterialTheme.colorScheme.onSurface,
            subtitleStyle = MaterialTheme.typography.listItemSubText,
            subtitleColor = MaterialTheme.colorScheme.onSurface,
            titleText = obfuscationMode.toTitle(),
            subtitleText = stringResource(id = R.string.port_x, port.toSubTitle()),
            onCellClicked = { onSelected(obfuscationMode) },
            minHeight = Dimens.cellHeight,
            background =
                if (isSelected) {
                    MaterialTheme.colorScheme.selected
                } else {
                    Color.Transparent
                },
            iconView = {
                SelectableIcon(
                    iconContentDescription = null,
                    isSelected = isSelected,
                    isEnabled = true,
                )
            },
        )
        VerticalDivider(
            color = MaterialTheme.colorScheme.surface,
            modifier = Modifier.fillMaxHeight().padding(vertical = Dimens.verticalDividerPadding),
        )

        Box(
            modifier =
                Modifier.widthIn(min = Dimens.obfuscationNavigationBoxWidth)
                    .fillMaxHeight()
                    .clickable { onNavigate() },
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = Icons.Default.ChevronRight,
                tint = MaterialTheme.colorScheme.onPrimary,
                contentDescription = null,
            )
        }
    }
}

@Composable
private fun ObfuscationMode.toTitle() =
    when (this) {
        ObfuscationMode.Auto -> stringResource(id = R.string.automatic)
        ObfuscationMode.Off -> stringResource(id = R.string.off)
        ObfuscationMode.Udp2Tcp -> stringResource(id = R.string.upd_over_tcp)
        ObfuscationMode.Shadowsocks -> stringResource(id = R.string.shadowsocks)
    }

@Composable
private fun Constraint<Port>.toSubTitle() =
    when (this) {
        Constraint.Any -> stringResource(id = R.string.automatic)
        is Constraint.Only -> this.value.toString()
    }

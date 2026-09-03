package net.mullvad.mullvadvpn.lib.ui.designsystem

import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Clear
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.InputChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.ui.designsystem.preview.PreviewColumn
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.ui.theme.color.primaryDisabled
import net.mullvad.mullvadvpn.lib.ui.theme.shape.chipShape

@Preview
@Composable
private fun PreviewEnabledMullvadFilterChip() = PreviewColumn {
    MullvadFilterChip(text = "Providers: 10", onClick = {}, enabled = true)
}

@Preview
@Composable
private fun PreviewDisabledMullvadFilterChip() = PreviewColumn {
    MullvadFilterChip(text = "Providers: 17", onClick = {}, enabled = false)
}

@Preview
@Composable
private fun PreviewInactiveMullvadFilterChip() = PreviewColumn {
    MullvadFilterChip(text = "Providers: 17", onClick = {}, enabled = true, active = false)
}

@Composable
fun MullvadFilterChip(
    containerColor: Color = MaterialTheme.colorScheme.primaryContainer,
    borderColor: Color = Color.Transparent,
    labelColor: Color = MaterialTheme.colorScheme.onPrimary,
    iconColor: Color = MaterialTheme.colorScheme.onPrimary,
    text: String,
    onClick: () -> Unit,
    showCrossIcon: Boolean = false,
    enabled: Boolean = true,
    active: Boolean = true,
) {
    InputChip(
        enabled = enabled,
        shape = MaterialTheme.shapes.chipShape,
        colors =
            FilterChipDefaults.filterChipColors(
                containerColor =
                    if (active) containerColor else MaterialTheme.colorScheme.primaryDisabled,
                disabledContainerColor = containerColor,
                labelColor = if (active) labelColor else labelColor.copy(alpha = AlphaInactive),
                disabledLabelColor = labelColor,
                iconColor = iconColor,
            ),
        border =
            FilterChipDefaults.filterChipBorder(
                borderColor = borderColor,
                disabledBorderColor = borderColor,
                enabled = true,
                selected = false,
            ),
        selected = false,
        onClick = onClick,
        label = { Text(text = text, style = MaterialTheme.typography.labelLarge) },
        trailingIcon =
            if (showCrossIcon) {
                {
                    Icon(
                        imageVector = Icons.Rounded.Clear,
                        contentDescription = null,
                        modifier = Modifier.size(Dimens.smallIconSize),
                    )
                }
            } else null,
    )
}

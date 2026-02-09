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
import net.mullvad.mullvadvpn.lib.ui.theme.shape.chipShape

@Preview
@Composable
private fun PreviewEnabledMullvadFilterChip() = PreviewColumn {
    MullvadFilterChip(text = "Providers: 10", onRemoveClick = {}, enabled = true)
}

@Preview
@Composable
private fun PreviewDisabledMullvadFilterChip() = PreviewColumn {
    MullvadFilterChip(text = "Providers: 17", onRemoveClick = {}, enabled = false)
}

@Composable
fun MullvadFilterChip(
    containerColor: Color = MaterialTheme.colorScheme.primary,
    borderColor: Color = Color.Transparent,
    labelColor: Color = MaterialTheme.colorScheme.onPrimary,
    iconColor: Color = MaterialTheme.colorScheme.onPrimary,
    text: String,
    onRemoveClick: () -> Unit,
    enabled: Boolean,
) {
    InputChip(
        enabled = enabled,
        shape = MaterialTheme.shapes.chipShape,
        colors =
            FilterChipDefaults.filterChipColors(
                containerColor = containerColor,
                disabledContainerColor = containerColor,
                labelColor = labelColor,
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
        onClick = onRemoveClick,
        label = { Text(text = text, style = MaterialTheme.typography.labelLarge) },
        trailingIcon =
            if (enabled) {
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

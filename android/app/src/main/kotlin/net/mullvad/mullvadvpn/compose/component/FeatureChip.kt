package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.shape.chipShape

@Preview
@Composable
private fun PreviewMullvadFeatureChip() {
    AppTheme { Row { MullvadFeatureChip(text = "DAITA") } }
}

@Composable
fun MullvadFeatureChip(
    containerColor: Color = MaterialTheme.colorScheme.surfaceContainerLowest,
    borderColor: Color = MaterialTheme.colorScheme.primary,
    labelColor: Color = MaterialTheme.colorScheme.onPrimary,
    iconColor: Color = MaterialTheme.colorScheme.onPrimary,
    text: String,
) {
    FilterChip(
        shape = MaterialTheme.shapes.chipShape,
        colors =
            FilterChipDefaults.filterChipColors(
                disabledContainerColor = containerColor,
                disabledLabelColor = labelColor,
                labelColor = labelColor,
                iconColor = iconColor,
            ),
        border =
            FilterChipDefaults.filterChipBorder(
                borderColor = borderColor,
                enabled = true,
                selected = false,
            ),
        selected = false,
        onClick = {},
        enabled = false,
        label = { Text(text = text, style = MaterialTheme.typography.labelMedium) },
    )
}

@Composable
fun MullvadMoreChip(
    onClick: () -> Unit,
    containerColor: Color = MaterialTheme.colorScheme.background,
    borderColor: Color = Color.Transparent,
    labelColor: Color = MaterialTheme.colorScheme.onPrimary,
    iconColor: Color = MaterialTheme.colorScheme.onPrimary,
    text: String,
) {
    FilterChip(
        onClick = onClick,
        shape = MaterialTheme.shapes.chipShape,
        colors =
            FilterChipDefaults.filterChipColors(
                containerColor = containerColor,
                labelColor = labelColor,
                iconColor = iconColor,
            ),
        border =
            FilterChipDefaults.filterChipBorder(
                borderColor = borderColor,
                enabled = true,
                selected = false,
            ),
        selected = false,
        label = { Text(text = text, style = MaterialTheme.typography.labelMedium) },
    )
}

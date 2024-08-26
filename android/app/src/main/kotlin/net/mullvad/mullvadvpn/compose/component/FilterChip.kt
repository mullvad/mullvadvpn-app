package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.size
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.InputChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.shape.chipShape

@Preview
@Composable
private fun PreviewEnabledMullvadFilterChip() {
    AppTheme {
        MullvadFilterChip(
            text = stringResource(id = R.string.number_of_providers),
            onRemoveClick = {},
            enabled = true,
        )
    }
}

@Preview
@Composable
private fun PreviewDisabledMullvadFilterChip() {
    AppTheme {
        MullvadFilterChip(
            text = stringResource(id = R.string.number_of_providers),
            onRemoveClick = {},
            enabled = false,
        )
    }
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
        label = { Text(text = text, style = MaterialTheme.typography.labelMedium) },
        trailingIcon =
            if (enabled) {
                {
                    Icon(
                        painter = painterResource(id = R.drawable.icon_close),
                        contentDescription = null,
                        modifier = Modifier.size(Dimens.smallIconSize),
                    )
                }
            } else null,
    )
}

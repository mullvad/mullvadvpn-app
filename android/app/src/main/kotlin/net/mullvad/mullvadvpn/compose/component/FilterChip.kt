package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
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
import net.mullvad.mullvadvpn.lib.theme.color.MullvadBlue
import net.mullvad.mullvadvpn.lib.theme.shape.chipShape

@Preview
@Composable
private fun PreviewMullvadFilterChip() {
    AppTheme {
        MullvadFilterChip(
            text = stringResource(id = R.string.number_of_providers),
            onRemoveClick = {}
        )
    }
}

@Composable
fun MullvadFilterChip(text: String, onRemoveClick: () -> Unit) {
    FilterChip(
        modifier = Modifier.padding(vertical = Dimens.chipVerticalPadding),
        shape = MaterialTheme.shapes.chipShape,
        colors = FilterChipDefaults.filterChipColors(containerColor = MullvadBlue),
        border =
            FilterChipDefaults.filterChipBorder(
                borderColor = Color.Transparent,
                disabledBorderColor = Color.Transparent,
                enabled = true,
                selected = false
            ),
        selected = false,
        onClick = {},
        label = {
            Text(
                text = text,
                color = MaterialTheme.colorScheme.onPrimary,
                style = MaterialTheme.typography.labelMedium
            )
            Spacer(modifier = Modifier.size(ButtonDefaults.IconSpacing))
            Image(
                painter = painterResource(id = R.drawable.icon_close),
                contentDescription = null,
                modifier = Modifier.size(Dimens.smallIconSize).clickable { onRemoveClick() }
            )
        }
    )
}

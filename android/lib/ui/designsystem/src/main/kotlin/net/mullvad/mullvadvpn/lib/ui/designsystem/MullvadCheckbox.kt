package net.mullvad.mullvadvpn.lib.ui.designsystem

import androidx.compose.foundation.background
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxColors
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.selected

@Preview
@Composable
private fun PreviewMullvadCheckbox() {
    AppTheme {
        Column(
            Modifier.background(color = MaterialTheme.colorScheme.background),
            verticalArrangement = Arrangement.spacedBy(Dimens.smallSpacer),
        ) {
            MullvadCheckbox(checked = false, null)
            MullvadCheckbox(checked = true, null)
            MullvadCheckbox(checked = false, null, enabled = false)
            MullvadCheckbox(checked = true, null, enabled = false)
        }
    }
}

@Composable
fun MullvadCheckbox(
    checked: Boolean,
    onCheckedChange: ((Boolean) -> Unit)?,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    colors: CheckboxColors =
        CheckboxDefaults.colors(
            checkedColor = MaterialTheme.colorScheme.onPrimary,
            uncheckedColor = MaterialTheme.colorScheme.onPrimary,
            checkmarkColor = MaterialTheme.colorScheme.selected,
        ),
    interactionSource: MutableInteractionSource? = null,
) {
    Checkbox(
        checked = checked,
        onCheckedChange = onCheckedChange,
        modifier = modifier,
        enabled = enabled,
        colors = colors,
        interactionSource = interactionSource,
    )
}

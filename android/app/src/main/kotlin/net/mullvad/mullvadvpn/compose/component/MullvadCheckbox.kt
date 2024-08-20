package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.selected

@Preview
@Composable
private fun PreviewMullvadCheckbox() {
    AppTheme {
        SpacedColumn(Modifier.background(color = MaterialTheme.colorScheme.primary)) {
            MullvadCheckbox(checked = false) {}
            MullvadCheckbox(checked = true) {}
        }
    }
}

@Composable
fun MullvadCheckbox(
    checkedColor: Color = MaterialTheme.colorScheme.onPrimary,
    uncheckedColor: Color = MaterialTheme.colorScheme.onPrimary,
    checkmarkColor: Color = MaterialTheme.colorScheme.selected,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit
) {
    Checkbox(
        checked = checked,
        onCheckedChange = onCheckedChange,
        colors =
            CheckboxDefaults.colors(
                checkedColor = checkedColor,
                uncheckedColor = uncheckedColor,
                checkmarkColor = checkmarkColor
            )
    )
}

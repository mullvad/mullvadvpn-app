package net.mullvad.mullvadvpn.compose.component

import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import net.mullvad.mullvadvpn.lib.theme.color.selected

@Composable
fun MullvadCheckbox(checked: Boolean, onCheckedChange: (Boolean) -> Unit) {
    Checkbox(
        checked = checked,
        onCheckedChange = onCheckedChange,
        colors =
            CheckboxDefaults.colors(
                checkedColor = MaterialTheme.colorScheme.onPrimary,
                uncheckedColor = MaterialTheme.colorScheme.onPrimary,
                checkmarkColor = MaterialTheme.colorScheme.selected
            )
    )
}

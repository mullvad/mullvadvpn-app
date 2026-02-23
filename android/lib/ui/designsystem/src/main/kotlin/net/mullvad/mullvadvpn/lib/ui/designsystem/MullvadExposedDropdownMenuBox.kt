package net.mullvad.mullvadvpn.lib.ui.designsystem

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuAnchorType
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldColors
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextStyle
import net.mullvad.mullvadvpn.lib.ui.theme.color.menuItemColors

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MullvadExposedDropdownMenuBox(
    modifier: Modifier = Modifier,
    label: String,
    title: String,
    colors: TextFieldColors,
    content: @Composable ColumnScope.(onClick: () -> Unit) -> Unit,
) {
    var expanded by remember { mutableStateOf(false) }
    ExposedDropdownMenuBox(
        expanded = expanded,
        onExpandedChange = { expanded = it },
        modifier = modifier.clickable { expanded = !expanded },
    ) {
        TextField(
            modifier =
                Modifier.fillMaxWidth()
                    .menuAnchor(ExposedDropdownMenuAnchorType.PrimaryNotEditable, true),
            readOnly = true,
            value = title,
            onValueChange = { /* Do nothing */ },
            label = { Text(text = label) },
            trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = expanded) },
            colors = colors,
        )
        ExposedDropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            modifier = Modifier.background(MaterialTheme.colorScheme.secondaryContainer),
        ) {
            content { expanded = false }
        }
    }
}

@Composable
fun MullvadDropdownMenuItem(
    text: String,
    style: TextStyle = MaterialTheme.typography.bodyLarge,
    onClick: () -> Unit,
    content: @Composable (() -> Unit)? = null,
) {
    DropdownMenuItem(
        leadingIcon = content,
        colors = menuItemColors,
        text = { Text(text = text, style = style) },
        onClick = onClick,
    )
}

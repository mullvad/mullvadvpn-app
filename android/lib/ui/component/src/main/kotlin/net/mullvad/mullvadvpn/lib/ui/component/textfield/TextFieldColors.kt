package net.mullvad.mullvadvpn.lib.ui.component.textfield

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.selection.TextSelectionColors
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldColors
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.color.Alpha20
import net.mullvad.mullvadvpn.lib.ui.theme.color.Alpha40
import net.mullvad.mullvadvpn.lib.ui.theme.color.Alpha5
import net.mullvad.mullvadvpn.lib.ui.theme.color.Alpha60

@Composable
fun mullvadWhiteTextFieldColors(): TextFieldColors =
    TextFieldDefaults.colors(
        focusedTextColor = Color.Black,
        unfocusedTextColor = Color.Gray,
        disabledTextColor = Color.Gray,
        disabledContainerColor = MaterialTheme.colorScheme.inverseSurface,
        errorContainerColor = MaterialTheme.colorScheme.inverseSurface,
        focusedContainerColor = MaterialTheme.colorScheme.inverseSurface,
        unfocusedContainerColor = MaterialTheme.colorScheme.inverseSurface,
        errorTextColor = Color.Black,
        cursorColor = MaterialTheme.colorScheme.background,
        focusedPlaceholderColor = MaterialTheme.colorScheme.background,
        unfocusedPlaceholderColor = MaterialTheme.colorScheme.primary.copy(alpha = Alpha40),
        focusedLabelColor = MaterialTheme.colorScheme.background,
        disabledLabelColor = Color.Gray,
        unfocusedLabelColor = MaterialTheme.colorScheme.primary.copy(alpha = Alpha40),
        focusedLeadingIconColor = Color.Black,
        unfocusedSupportingTextColor = Color.Black,
        focusedIndicatorColor = MaterialTheme.colorScheme.onPrimary,
        unfocusedIndicatorColor = MaterialTheme.colorScheme.onPrimary,
        errorIndicatorColor = MaterialTheme.colorScheme.error,
    )

@Preview
@Composable
private fun PreviewDarkTextField() {
    AppTheme {
        Column(
            modifier = Modifier.background(MaterialTheme.colorScheme.background).padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            // Normal
            TextField(
                value = "Value",
                onValueChange = {},
                placeholder = { Text(text = "Placeholder") },
                label = { Text(text = "Label") },
                colors = mullvadDarkTextFieldColors(),
                shape = MaterialTheme.shapes.small,
            )

            // Normal empty
            TextField(
                value = "",
                onValueChange = {},
                placeholder = { Text(text = "Placeholder") },
                label = { Text(text = "Label") },
                colors = mullvadDarkTextFieldColors(),
                shape = MaterialTheme.shapes.small,
            )

            // Error
            TextField(
                value = "Value",
                onValueChange = {},
                placeholder = { Text(text = "Placeholder") },
                label = { Text(text = "Label") },
                isError = true,
                colors = mullvadDarkTextFieldColors(),
                shape = MaterialTheme.shapes.small,
            )
        }
    }
}

@Composable
fun mullvadDarkTextFieldColors(): TextFieldColors =
    TextFieldDefaults.colors(
        focusedTextColor = MaterialTheme.colorScheme.onSurface,
        unfocusedTextColor = MaterialTheme.colorScheme.onSurface,
        disabledTextColor = MaterialTheme.colorScheme.onSurface.copy(Alpha20),
        disabledContainerColor =
            MaterialTheme.colorScheme.onSurface
                .copy(alpha = Alpha5)
                .compositeOver(MaterialTheme.colorScheme.surfaceContainer),
        errorContainerColor = MaterialTheme.colorScheme.surfaceContainer,
        focusedContainerColor = MaterialTheme.colorScheme.surfaceContainer,
        unfocusedContainerColor = MaterialTheme.colorScheme.surfaceContainer,
        errorTextColor = MaterialTheme.colorScheme.onSurface,
        cursorColor = MaterialTheme.colorScheme.onSurface,
        focusedPlaceholderColor = MaterialTheme.colorScheme.onSurface.copy(Alpha60),
        unfocusedPlaceholderColor = MaterialTheme.colorScheme.onSurface.copy(Alpha60),
        focusedLabelColor = MaterialTheme.colorScheme.onSurface,
        disabledLabelColor = MaterialTheme.colorScheme.onSurface.copy(Alpha20),
        unfocusedLabelColor = MaterialTheme.colorScheme.onSurface,
        focusedLeadingIconColor = Color.Black,
        focusedSupportingTextColor = Color.Black,
        unfocusedSupportingTextColor = Color.Black,
        focusedIndicatorColor = MaterialTheme.colorScheme.onSurface,
        disabledIndicatorColor =
            MaterialTheme.colorScheme.onSurface
                .copy(alpha = Alpha20)
                .compositeOver(MaterialTheme.colorScheme.surfaceContainer),
        errorIndicatorColor = MaterialTheme.colorScheme.error,
        unfocusedIndicatorColor = MaterialTheme.colorScheme.onSurface.copy(alpha = Alpha40),
        selectionColors =
            TextSelectionColors(
                handleColor = MaterialTheme.colorScheme.primary,
                backgroundColor = MaterialTheme.colorScheme.tertiary,
            ),
    )

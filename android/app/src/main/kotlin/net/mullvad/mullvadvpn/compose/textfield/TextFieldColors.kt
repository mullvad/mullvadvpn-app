package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
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
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.Alpha20
import net.mullvad.mullvadvpn.lib.theme.color.Alpha40

@Composable
fun mullvadWhiteTextFieldColors(): TextFieldColors =
    TextFieldDefaults.colors(
        focusedTextColor = Color.Black,
        unfocusedTextColor = Color.Gray,
        disabledTextColor = Color.Gray,
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
        errorIndicatorColor = MaterialTheme.colorScheme.error
    )

@Preview
@Composable
private fun PreviewDarkTextField() {
    AppTheme {
        Column(
            modifier = Modifier.background(MaterialTheme.colorScheme.background).padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Normal
            TextField(
                value = "Value",
                onValueChange = {},
                placeholder = { Text(text = "Placeholder") },
                label = { Text(text = "Label") },
                colors = mullvadDarkTextFieldColors(),
                shape = MaterialTheme.shapes.small
            )

            // Normal empty
            TextField(
                value = "",
                onValueChange = {},
                placeholder = { Text(text = "Placeholder") },
                label = { Text(text = "Label") },
                colors = mullvadDarkTextFieldColors(),
                shape = MaterialTheme.shapes.small
            )

            // Error
            TextField(
                value = "Value",
                onValueChange = {},
                placeholder = { Text(text = "Placeholder") },
                label = { Text(text = "Label") },
                isError = true,
                colors = mullvadDarkTextFieldColors(),
                shape = MaterialTheme.shapes.small
            )
        }
    }
}

@Composable
fun mullvadDarkTextFieldColors(): TextFieldColors =
    TextFieldDefaults.colors(
        focusedTextColor = MaterialTheme.colorScheme.primary,
        unfocusedTextColor = MaterialTheme.colorScheme.onPrimary,
        disabledTextColor = MaterialTheme.colorScheme.onPrimary,
        disabledContainerColor = MaterialTheme.colorScheme.primary,
        errorContainerColor = MaterialTheme.colorScheme.onPrimary,
        focusedContainerColor = MaterialTheme.colorScheme.onPrimary,
        unfocusedContainerColor =
            MaterialTheme.colorScheme.onPrimary
                .copy(alpha = Alpha20)
                .compositeOver(MaterialTheme.colorScheme.primary),
        errorTextColor = MaterialTheme.colorScheme.error,
        cursorColor = MaterialTheme.colorScheme.background,
        focusedPlaceholderColor = MaterialTheme.colorScheme.primary,
        unfocusedPlaceholderColor = MaterialTheme.colorScheme.onPrimary,
        focusedLabelColor = MaterialTheme.colorScheme.primary,
        disabledLabelColor = Color.Gray,
        unfocusedLabelColor = MaterialTheme.colorScheme.onPrimary,
        focusedLeadingIconColor = Color.Black,
        focusedSupportingTextColor = Color.Black,
        unfocusedSupportingTextColor = Color.Black,
        focusedIndicatorColor = Color.Transparent,
        disabledIndicatorColor = Color.Transparent,
        errorIndicatorColor = Color.Transparent,
        unfocusedIndicatorColor = Color.Transparent,
    )

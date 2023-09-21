package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.TextFieldColors
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

@Composable
fun mullvadWhiteTextFieldColors(): TextFieldColors =
    TextFieldDefaults.colors(
        focusedTextColor = Color.Black,
        unfocusedTextColor = Color.Gray,
        disabledTextColor = Color.Gray,
        errorTextColor = Color.Black,
        cursorColor = MaterialTheme.colorScheme.background,
        focusedPlaceholderColor = MaterialTheme.colorScheme.background,
        unfocusedPlaceholderColor = MaterialTheme.colorScheme.primary,
        focusedLabelColor = MaterialTheme.colorScheme.background,
        disabledLabelColor = Color.Gray,
        unfocusedLabelColor = MaterialTheme.colorScheme.background,
        focusedLeadingIconColor = Color.Black,
        unfocusedSupportingTextColor = Color.Black,
    )

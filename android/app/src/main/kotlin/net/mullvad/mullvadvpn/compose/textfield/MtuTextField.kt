package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType

@Composable
fun MtuTextField(
    value: String,
    isValidValue: Boolean,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit = {},
    onSubmit: (String) -> Unit = {},
    isEnabled: Boolean = true,
    placeholderText: String = "",
    maxCharLength: Int
) {
    CustomTextField(
        value = value,
        keyboardType = KeyboardType.Number,
        modifier = modifier,
        onValueChanged = onValueChanged,
        onSubmit = onSubmit,
        isEnabled = isEnabled,
        placeholderText = placeholderText,
        maxCharLength = maxCharLength,
        isValidValue = isValidValue,
        isDigitsOnlyAllowed = true
    )
}

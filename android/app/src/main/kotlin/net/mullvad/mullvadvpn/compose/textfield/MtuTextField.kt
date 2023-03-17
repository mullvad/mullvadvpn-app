package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

@Composable
fun MtuTextField(
    value: String,
    isValidValue: Boolean,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit = { },
    onFocusChange: (Boolean) -> Unit = { },
    onSubmit: (String) -> Unit = { },
    isEnabled: Boolean = true,
    placeholderText: String = "",
    maxCharLength: Int
) {
    CustomTextField(
        value = value,
        modifier = modifier,
        onValueChanged = onValueChanged,
        onFocusChange = onFocusChange,
        onSubmit = onSubmit,
        isEnabled = isEnabled,
        placeholderText = placeholderText,
        maxCharLength = maxCharLength,
        isValidValue = isValidValue,
        isDigitsOnlyAllowed = true
    )
}

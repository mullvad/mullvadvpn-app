package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign

@Composable
fun DnsTextField(
    value: String,
    isValidValue: Boolean,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit = {},
    onFocusChanges: (Boolean) -> Unit = {},
    onSubmit: (String) -> Unit = {},
    placeholderText: String = "",
    isEnabled: Boolean = true
) {
    CustomTextField(
        value = value,
        modifier = modifier,
        onValueChanged = onValueChanged,
        onFocusChange = onFocusChanges,
        onSubmit = onSubmit,
        isEnabled = isEnabled,
        placeholderText = placeholderText,
        maxCharLength = Int.MAX_VALUE,
        isValidValue = isValidValue,
        isDigitsOnlyAllowed = false,
        textAlign = TextAlign.Start
    )
}

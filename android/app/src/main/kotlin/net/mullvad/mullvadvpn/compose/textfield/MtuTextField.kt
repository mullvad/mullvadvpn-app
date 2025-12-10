package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextDirection

@Composable
fun MtuTextField(
    value: String,
    isValidValue: Boolean,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit = {},
    onSubmit: (String) -> Unit = {},
    isEnabled: Boolean = true,
    placeholderText: String = "",
    maxCharLength: Int,
) {
    CustomTextField(
        value = value,
        keyboardType = KeyboardType.Number,
        modifier = modifier,
        onValueChanged = onValueChanged,
        onSubmit = onSubmit,
        isEnabled = isEnabled,
        placeholder = {
            Text(text = placeholderText, style = MaterialTheme.typography.titleMedium)
        },
        maxCharLength = maxCharLength,
        isValidValue = isValidValue,
        isDigitsOnlyAllowed = true,
        textStyle = MaterialTheme.typography.titleMedium.copy(textDirection = TextDirection.Ltr),
    )
}

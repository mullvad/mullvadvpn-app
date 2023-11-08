package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.VisualTransformation

@Composable
fun GroupedTextField(
    value: String,
    keyboardType: KeyboardType,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit,
    onSubmit: (String) -> Unit,
    isEnabled: Boolean = true,
    visualTransformation: VisualTransformation,
    placeholderText: String = "",
    maxCharLength: Int = Int.MAX_VALUE,
    isValidValue: Boolean,
    isDigitsOnlyAllowed: Boolean,
    validateRegex: Regex?,
) {
    CustomTextField(
        value = value,
        keyboardType = keyboardType,
        onValueChanged = {
            if (validateRegex == null || validateRegex.matches(it)) onValueChanged(it)
        },
        onSubmit = onSubmit,
        isDigitsOnlyAllowed = isDigitsOnlyAllowed,
        modifier = modifier,
        isEnabled = isEnabled,
        visualTransformation = visualTransformation,
        placeholderText = placeholderText,
        maxCharLength = maxCharLength,
        isValidValue = isValidValue,
    )
}

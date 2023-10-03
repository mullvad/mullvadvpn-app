package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign

@Composable
fun GroupedTextField(
    value: String,
    keyboardType: KeyboardType,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit,
    onFocusChange: (Boolean) -> Unit,
    onSubmit: (String) -> Unit,
    isEnabled: Boolean = true,
    visualTransformation: VisualTransformation,
    placeholderText: String = "",
    placeHolderColor: Color = MaterialTheme.colorScheme.primary,
    maxCharLength: Int = Int.MAX_VALUE,
    isValidValue: Boolean,
    isDigitsOnlyAllowed: Boolean,
    validateRegex: Regex,
    defaultTextColor: Color = MaterialTheme.colorScheme.onPrimary,
    textAlign: TextAlign = TextAlign.Start
) {
    CustomTextField(
        value = value,
        keyboardType = keyboardType,
        onValueChanged = { if (validateRegex.matches(it)) onValueChanged(it) },
        onFocusChange = onFocusChange,
        onSubmit = onSubmit,
        isValidValue = isValidValue,
        isDigitsOnlyAllowed = isDigitsOnlyAllowed,
        modifier = modifier,
        isEnabled = isEnabled,
        visualTransformation = visualTransformation,
        placeholderText = placeholderText,
        placeHolderColor = placeHolderColor,
        maxCharLength = maxCharLength,
        defaultTextColor = defaultTextColor,
        textAlign = textAlign
    )
}

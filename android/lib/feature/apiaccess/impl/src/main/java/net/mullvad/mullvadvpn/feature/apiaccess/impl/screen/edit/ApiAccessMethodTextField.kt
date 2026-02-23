package net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.edit

import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import net.mullvad.mullvadvpn.lib.ui.component.textfield.CustomTextField
import net.mullvad.mullvadvpn.lib.ui.component.textfield.ErrorSupportingText
import net.mullvad.mullvadvpn.lib.ui.component.textfield.mullvadDarkTextFieldColors

@Composable
fun ApiAccessMethodTextField(
    value: String,
    keyboardType: KeyboardType,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit,
    labelText: String?,
    maxCharLength: Int = Int.MAX_VALUE,
    isValidValue: Boolean,
    isDigitsOnlyAllowed: Boolean,
    errorText: String?,
    capitalization: KeyboardCapitalization = KeyboardCapitalization.None,
    imeAction: ImeAction = ImeAction.Next,
    textStyle: TextStyle = MaterialTheme.typography.bodyLarge,
) {
    val focusManager = LocalFocusManager.current
    CustomTextField(
        value = value,
        keyboardType = keyboardType,
        onValueChanged = onValueChanged,
        modifier = modifier,
        onSubmit = {
            if (imeAction == ImeAction.Done) {
                focusManager.clearFocus()
            }
        },
        labelText = labelText,
        maxCharLength = maxCharLength,
        isValidValue = isValidValue,
        isDigitsOnlyAllowed = isDigitsOnlyAllowed,
        colors = mullvadDarkTextFieldColors(),
        keyboardOptions =
            KeyboardOptions(
                capitalization = capitalization,
                autoCorrectEnabled = false,
                keyboardType = keyboardType,
                imeAction = imeAction,
            ),
        supportingText = errorText?.let { { ErrorSupportingText(errorText) } },
        textStyle = textStyle,
    )
}

package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import net.mullvad.mullvadvpn.lib.theme.Dimens

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
) {
    val focusManager = LocalFocusManager.current
    CustomTextField(
        value = value,
        keyboardType = keyboardType,
        onValueChanged = onValueChanged,
        onSubmit = {
            if (imeAction == ImeAction.Done) {
                focusManager.clearFocus()
            }
        },
        labelText = labelText,
        placeholderText = null,
        isValidValue = isValidValue,
        isDigitsOnlyAllowed = isDigitsOnlyAllowed,
        maxCharLength = maxCharLength,
        supportingText = errorText?.let { { ErrorSupportingText(errorText) } },
        colors = apiAccessTextFieldColors(),
        modifier =
            modifier
                .defaultMinSize(minHeight = Dimens.formTextFieldMinHeight)
                .padding(vertical = Dimens.miniPadding),
        keyboardOptions =
            KeyboardOptions(
                capitalization = capitalization,
                autoCorrectEnabled = false,
                keyboardType = keyboardType,
                imeAction = imeAction,
            ),
    )
}

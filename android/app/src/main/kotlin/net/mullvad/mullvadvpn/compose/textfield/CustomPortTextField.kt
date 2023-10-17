package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import net.mullvad.mullvadvpn.R

@Composable
fun CustomPortTextField(
    value: String,
    modifier: Modifier = Modifier,
    onSubmit: (String) -> Unit,
    onValueChanged: (String) -> Unit,
    isValidValue: Boolean,
    maxCharLength: Int
) {
    CustomTextField(
        value = value,
        keyboardType = KeyboardType.Number,
        modifier = modifier,
        placeholderText = stringResource(id = R.string.custom_port_dialog_placeholder),
        onValueChanged = onValueChanged,
        onSubmit = onSubmit,
        isDigitsOnlyAllowed = true,
        isEnabled = true,
        isValidValue = isValidValue,
        maxCharLength = maxCharLength
    )
}

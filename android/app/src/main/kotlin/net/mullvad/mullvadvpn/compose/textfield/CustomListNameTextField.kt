package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import net.mullvad.mullvadvpn.lib.model.CustomListName

@Composable
fun CustomListNameTextField(
    modifier: Modifier = Modifier,
    name: String,
    isValidName: Boolean,
    error: String?,
    onValueChanged: (String) -> Unit,
    onSubmit: (String) -> Unit,
) {
    val focusRequester = remember { FocusRequester() }
    val keyboardController = LocalSoftwareKeyboardController.current
    CustomTextField(
        value = name,
        onValueChanged = onValueChanged,
        onSubmit = {
            if (isValidName) {
                onSubmit(it)
            }
        },
        keyboardType = KeyboardType.Text,
        placeholderText = null,
        isValidValue = error == null,
        isDigitsOnlyAllowed = false,
        maxCharLength = CustomListName.MAX_LENGTH,
        supportingText =
            error?.let {
                {
                    Text(
                        text = it,
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            },
        capitalization = KeyboardCapitalization.Words,
        textStyle = MaterialTheme.typography.titleMedium,
        modifier =
            modifier.focusRequester(focusRequester).onFocusChanged { focusState ->
                if (focusState.hasFocus) {
                    keyboardController?.show()
                }
            },
    )

    LaunchedEffect(Unit) { focusRequester.requestFocus() }
}

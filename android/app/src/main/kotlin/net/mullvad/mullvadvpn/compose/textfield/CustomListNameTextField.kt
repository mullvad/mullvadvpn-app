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
        keyboardType = KeyboardType.Text,
        modifier =
            modifier.focusRequester(focusRequester).onFocusChanged { focusState ->
                if (focusState.hasFocus) {
                    keyboardController?.show()
                }
            },
        onValueChanged = onValueChanged,
        onSubmit = {
            if (isValidName) {
                onSubmit(it)
            }
        },
        maxCharLength = CustomListName.MAX_LENGTH,
        isValidValue = error == null,
        isDigitsOnlyAllowed = false,
        textStyle = MaterialTheme.typography.titleMedium,
        capitalization = KeyboardCapitalization.Words,
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
    )

    LaunchedEffect(Unit) { focusRequester.requestFocus() }
}

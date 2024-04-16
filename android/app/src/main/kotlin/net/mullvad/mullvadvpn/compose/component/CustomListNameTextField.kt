package net.mullvad.mullvadvpn.compose.component

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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.textfield.CustomTextField
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.CustomListsError

@Composable
fun CustomListNameTextField(
    modifier: Modifier = Modifier,
    name: String,
    isValidName: Boolean,
    error: CustomListsError?,
    onValueChanged: (String) -> Unit,
    onSubmit: (String) -> Unit
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
        // This can not be set to KeyboardType.Text because it will show the
        // suggestions, this will cause an infinite loop on Android TV with Gboard
        keyboardType = KeyboardType.Password,
        placeholderText = null,
        isValidValue = error == null,
        isDigitsOnlyAllowed = false,
        maxCharLength = CustomListName.MAX_LENGTH,
        supportingText =
            error?.let {
                {
                    Text(
                        text = it.errorString(),
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall
                    )
                }
            },
        modifier =
            modifier.focusRequester(focusRequester).onFocusChanged { focusState ->
                if (focusState.hasFocus) {
                    keyboardController?.show()
                }
            }
    )

    LaunchedEffect(Unit) { focusRequester.requestFocus() }
}

@Composable
private fun CustomListsError.errorString() =
    stringResource(
        when (this) {
            CustomListsError.CustomListExists -> R.string.custom_list_error_list_exists
            CustomListsError.OtherError -> R.string.error_occurred
        }
    )

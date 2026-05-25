package net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.edit

import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.InputTransformation
import androidx.compose.foundation.text.input.byValue
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.core.text.isDigitsOnly
import kotlinx.coroutines.flow.collectLatest
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
    val textFieldState = rememberTextFieldState(value)
    LaunchedEffect(textFieldState) {
        snapshotFlow { textFieldState.text.toString() }.collectLatest { onValueChanged(it) }
    }
    TextField(
        state = textFieldState,
        keyboardOptions =
            KeyboardOptions(
                capitalization = capitalization,
                keyboardType = keyboardType,
                imeAction = imeAction,
                autoCorrectEnabled = false,
            ),
        textStyle = textStyle,
        modifier = modifier,
        colors = mullvadDarkTextFieldColors(),
        label =
            labelText?.let { { Text(text = it, maxLines = 1, overflow = TextOverflow.Ellipsis) } },
        supportingText = errorText?.let { { ErrorSupportingText(errorText) } },
        isError = !isValidValue,
        inputTransformation =
            InputTransformation.byValue { current, proposed ->
                when {
                    proposed.length > maxCharLength -> return@byValue current
                    proposed.isDigitsOnly().not() && isDigitsOnlyAllowed -> return@byValue current
                    else -> proposed
                }
            },
        onKeyboardAction = {
            if (imeAction == ImeAction.Done) {
                focusManager.clearFocus()
            }
        },
    )
}

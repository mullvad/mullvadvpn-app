package net.mullvad.mullvadvpn.compose.textfield

import android.text.TextUtils
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.VisualTransformation
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

private const val EMPTY_STRING = ""
private const val NEWLINE_STRING = "\n"

@Composable
fun CustomTextField(
    value: String,
    keyboardType: KeyboardType,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit,
    onSubmit: (String) -> Unit,
    isEnabled: Boolean = true,
    placeholderText: String?,
    maxCharLength: Int = Int.MAX_VALUE,
    isValidValue: Boolean,
    isDigitsOnlyAllowed: Boolean,
    visualTransformation: VisualTransformation = VisualTransformation.None
) {

    val scope = rememberCoroutineScope()

    // Pass initial text range ensure cursor position is correct when entering a TextField with a
    // preexisting value
    val textRange = remember { mutableStateOf(TextRange(value.length)) }

    TextField(
        value = TextFieldValue(value, textRange.value),
        onValueChange = { input ->
            val isValidInput = if (isDigitsOnlyAllowed) TextUtils.isDigitsOnly(input.text) else true
            if (input.text.length <= maxCharLength && isValidInput) {
                // Remove any newline chars added by enter key clicks
                textRange.value = input.selection
                onValueChanged(input.text.replace(NEWLINE_STRING, EMPTY_STRING))
            }
        },
        enabled = isEnabled,
        singleLine = true,
        placeholder = placeholderText?.let { { Text(text = it) } },
        keyboardOptions =
            KeyboardOptions(
                keyboardType = keyboardType,
                imeAction = ImeAction.Done,
                autoCorrect = false,
            ),
        keyboardActions =
            KeyboardActions(
                onDone = {
                    scope.launch {
                        // https://issuetracker.google.com/issues/305518328
                        delay(100)
                        onSubmit(value)
                    }
                }
            ),
        visualTransformation = visualTransformation,
        colors = mullvadDarkTextFieldColors(),
        isError = !isValidValue,
        modifier = modifier.clip(MaterialTheme.shapes.small).fillMaxWidth()
    )
}

package net.mullvad.mullvadvpn.compose.textfield

import android.text.TextUtils
import android.view.KeyEvent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusDirection
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AlphaInactive

private const val EMPTY_STRING = ""
private const val NEWLINE_STRING = "\n"

@Composable
@OptIn(ExperimentalComposeUiApi::class)
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
    val shape = RoundedCornerShape(4.dp)
    val textFieldHeight = 44.dp

    val focusManager = LocalFocusManager.current
    val keyboardController = LocalSoftwareKeyboardController.current

    var isFocused by remember { mutableStateOf(false) }

    val textColor =
        when {
            isValidValue.not() -> MaterialTheme.colorScheme.error
            isFocused -> MaterialTheme.colorScheme.primary
            else -> defaultTextColor
        }

    val placeholderTextColor =
        if (isFocused) {
            placeHolderColor
        } else {
            MaterialTheme.colorScheme.onPrimary
        }

    val backgroundColor =
        if (isFocused) {
            MaterialTheme.colorScheme.onPrimary
        } else {
            MaterialTheme.colorScheme.onPrimary.copy(AlphaInactive)
        }

    fun triggerSubmit() {
        keyboardController?.hide()
        focusManager.moveFocus(FocusDirection.Previous)
        onSubmit(value)
    }

    FilteredTextField(
        value = value,
        onChanged = { input ->
            val isValidInput = if (isDigitsOnlyAllowed) TextUtils.isDigitsOnly(input) else true
            if (input.length <= maxCharLength && isValidInput) {
                // Remove any newline chars added by enter key clicks
                onValueChanged(input.replace(NEWLINE_STRING, EMPTY_STRING))
            }
        },
        textStyle = MaterialTheme.typography.titleMedium.copy(color = textColor),
        isEnabled = isEnabled,
        singleLine = true,
        maxLines = 1,
        visualTransformation = visualTransformation,
        keyboardOptions =
            KeyboardOptions(
                keyboardType = keyboardType,
                imeAction = ImeAction.Done,
                autoCorrect = false,
            ),
        keyboardActions = KeyboardActions(onDone = { triggerSubmit() }),
        decorationBox = { decorationBox ->
            Box(modifier = Modifier.padding(PaddingValues(12.dp, 10.dp)).fillMaxWidth()) {
                if (value.isBlank()) {
                    Text(
                        text = placeholderText,
                        style = MaterialTheme.typography.titleMedium,
                        color = placeholderTextColor,
                        textAlign = textAlign,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
                decorationBox()
            }
        },
        cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
        validateRegex = validateRegex,
        modifier =
            modifier
                .background(backgroundColor)
                .clip(shape)
                .onFocusChanged { focusState ->
                    isFocused = focusState.isFocused
                    onFocusChange(focusState.isFocused)
                }
                .height(textFieldHeight)
                .onKeyEvent { keyEvent ->
                    return@onKeyEvent when (keyEvent.nativeKeyEvent.keyCode) {
                        KeyEvent.KEYCODE_ENTER -> {
                            triggerSubmit()
                            true
                        }
                        KeyEvent.KEYCODE_ESCAPE -> {
                            focusManager.clearFocus(force = true)
                            keyboardController?.hide()
                            true
                        }
                        KeyEvent.KEYCODE_DPAD_DOWN -> {
                            focusManager.moveFocus(FocusDirection.Down)
                            true
                        }
                        KeyEvent.KEYCODE_DPAD_UP -> {
                            focusManager.moveFocus(FocusDirection.Up)
                            true
                        }
                        else -> {
                            false
                        }
                    }
                }
    )
}

@Composable
fun FilteredTextField(
    value: String,
    onChanged: (String) -> Unit,
    isEnabled: Boolean,
    singleLine: Boolean,
    maxLines: Int,
    visualTransformation: VisualTransformation,
    textStyle: TextStyle,
    cursorBrush: Brush,
    keyboardOptions: KeyboardOptions,
    keyboardActions: KeyboardActions,
    validateRegex: Regex,
    modifier: Modifier = Modifier,
    decorationBox: @Composable (innerTextField: @Composable () -> Unit) -> Unit =
        @Composable { innerTextField -> innerTextField() }
) {
    BasicTextField(
        value = value,
        enabled = isEnabled,
        singleLine = singleLine,
        maxLines = maxLines,
        visualTransformation = visualTransformation,
        textStyle = textStyle,
        keyboardOptions = keyboardOptions,
        keyboardActions = keyboardActions,
        decorationBox = decorationBox,
        cursorBrush = cursorBrush,
        modifier = modifier,
        onValueChange = { if (validateRegex.matches(it)) onChanged(it) },
    )
}

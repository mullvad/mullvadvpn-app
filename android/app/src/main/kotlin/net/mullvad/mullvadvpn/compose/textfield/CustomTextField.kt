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
import androidx.compose.material.Text
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite10

private const val EMPTY_STRING = ""
private const val NEWLINE_STRING = "\n"

@Composable
@OptIn(ExperimentalComposeUiApi::class)
fun CustomTextField(
    value: String,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit,
    onFocusChange: (Boolean) -> Unit,
    onSubmit: (String) -> Unit,
    isEnabled: Boolean = true,
    placeholderText: String = "",
    placeHolderColor: Color = MullvadBlue,
    maxCharLength: Int = Int.MAX_VALUE,
    isValidValue: Boolean,
    isDigitsOnlyAllowed: Boolean,
    defaultTextColor: Color = Color.White,
    textAlign: TextAlign = TextAlign.Start
) {
    val fontSize = dimensionResource(id = R.dimen.text_medium_plus).value.sp
    val shape = RoundedCornerShape(4.dp)
    val textFieldHeight = 44.dp

    val focusManager = LocalFocusManager.current
    val keyboardController = LocalSoftwareKeyboardController.current

    var isFocused by remember { mutableStateOf(false) }

    val textColor =
        when {
            isValidValue.not() -> Color.Red
            isFocused -> MullvadBlue
            else -> defaultTextColor
        }

    val placeholderTextColor =
        if (isFocused) {
            placeHolderColor
        } else {
            Color.White
        }

    val backgroundColor =
        if (isFocused) {
            Color.White
        } else {
            MullvadWhite10
        }

    fun triggerSubmit() {
        keyboardController?.hide()
        focusManager.moveFocus(FocusDirection.Previous)
        onSubmit(value)
    }

    BasicTextField(
        value = value,
        onValueChange = { input ->
            val isValidInput = if (isDigitsOnlyAllowed) TextUtils.isDigitsOnly(input) else true
            if (input.length <= maxCharLength && isValidInput) {
                // Remove any newline chars added by enter key clicks
                onValueChanged(input.replace(NEWLINE_STRING, EMPTY_STRING))
            }
        },
        textStyle = TextStyle(color = textColor, fontSize = fontSize, textAlign = textAlign),
        enabled = isEnabled,
        singleLine = true,
        maxLines = 1,
        keyboardOptions =
            KeyboardOptions(
                keyboardType = KeyboardType.Number,
                imeAction = ImeAction.Done,
                autoCorrect = false,
            ),
        keyboardActions = KeyboardActions(onDone = { triggerSubmit() }),
        decorationBox = { decorationBox ->
            Box(modifier = Modifier.padding(PaddingValues(12.dp, 10.dp)).fillMaxWidth()) {
                if (value.isBlank()) {
                    Text(
                        text = placeholderText,
                        color = placeholderTextColor,
                        fontSize = fontSize,
                        textAlign = textAlign,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
                decorationBox()
            }
        },
        cursorBrush = SolidColor(MullvadBlue),
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

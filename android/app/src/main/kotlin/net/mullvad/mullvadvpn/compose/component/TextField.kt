package net.mullvad.mullvadvpn.compose.component

import android.text.TextUtils
import android.view.KeyEvent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
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
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite10

@Composable
fun CellTextField(
    value: String,
    onValueChanged: (String) -> Unit = { },
    onFocusChanges: (Boolean) -> Unit = { },
    onSubmit: (String) -> Unit = { },
    isEnabled: Boolean = true,
    placeholderText: String = "",
    maxCharLength: Int = Int.MAX_VALUE,
    // TODO: We might want to let the caller provide colors rather than using this validation func.
    isValidValue: (String) -> Boolean,
) {
    val modifier = Modifier
        .width(96.dp)
        .height(35.dp)
    CustomTextField(
        value = value,
        modifier = modifier,
        onValueChanged = onValueChanged,
        onFocusChanges = onFocusChanges,
        onSubmit = onSubmit,
        isEnabled = isEnabled,
        placeholderText = placeholderText,
        maxCharLength = maxCharLength,
        hasBackground = true,
        isValidValue = isValidValue,
        isValidChar = { TextUtils.isDigitsOnly(it) }
    )
}

@Composable
fun DnsTextField(
    value: String,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit = { },
    onFocusChanges: (Boolean) -> Unit = { },
    onSubmit: (String) -> Unit = { },
    isEnabled: Boolean = true,
    maxCharLength: Int = Int.MAX_VALUE,
    isValidValue: (String) -> Boolean,
) {
    CustomTextField(
        value = value,
        modifier = modifier,
        onValueChanged = onValueChanged,
        onFocusChanges = onFocusChanges,
        onSubmit = onSubmit,
        isEnabled = isEnabled,
        maxCharLength = maxCharLength,
        hasBackground = true,
        isValidValue = isValidValue,
        defaultTextColor = MullvadBlue,
        textAlign = TextAlign.Start
    )
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
private fun CustomTextField(
    value: String,
    modifier: Modifier = Modifier,
    onValueChanged: (String) -> Unit,
    onFocusChanges: (Boolean) -> Unit,
    onSubmit: (String) -> Unit,
    isEnabled: Boolean = true,
    placeholderText: String = "",
    maxCharLength: Int = Int.MAX_VALUE,
    hasBackground: Boolean = true,
    isValidValue: (String) -> Boolean,
    isValidChar: (String) -> Boolean = { true },
    defaultTextColor: Color = Color.White,
    textAlign: TextAlign = TextAlign.End
) {
    val fontSize = 18.sp
    val shape = RoundedCornerShape(4.dp)

    val focusManager = LocalFocusManager.current
    val keyboardController = LocalSoftwareKeyboardController.current

    val focusRequester = remember { FocusRequester() }
    var isFocused by remember { mutableStateOf(false) }

    val textColor = when {
        isValidValue(value).not() -> Color.Red
        isFocused -> MullvadBlue
        else -> defaultTextColor
    }

    val placeholderTextColor = if (isFocused) {
        MullvadBlue
    } else {
        Color.White
    }

    val backgroundColor = if (isFocused) {
        Color.White
    } else {
        MullvadWhite10
    }

    fun triggerSubmit() {
        keyboardController?.hide()
        focusManager.clearFocus()
        onSubmit(value)
    }

    var modifierTmp = modifier
        .clip(shape)
        .onFocusChanged { focusState ->
            onFocusChanges(focusState.isFocused)
        }
        .focusRequester(focusRequester)
        .onKeyEvent { keyEvent ->
            return@onKeyEvent when (keyEvent.nativeKeyEvent.keyCode) {
                KeyEvent.KEYCODE_ENTER -> {
                    triggerSubmit()
                    true
                }
                KeyEvent.KEYCODE_ESCAPE -> {
                    // TODO: Fix escape behavior!
                    focusManager.clearFocus(force = true)
                    keyboardController?.hide()
                    true
                }
                else -> false
            }
        }
    if (hasBackground) {
        modifierTmp = modifierTmp.background(backgroundColor)
    }

    BasicTextField(
        value = value,
        onValueChange = { input ->
            if (input.length <= maxCharLength && isValidChar(input)) {
                onValueChanged(input)
            }
        },
        modifier = modifierTmp,
        textStyle = TextStyle(
            color = textColor,
            fontSize = fontSize,
            textAlign = textAlign
        ),
        enabled = isEnabled,
        singleLine = true,
        maxLines = 1,
        keyboardOptions = KeyboardOptions(
            keyboardType = KeyboardType.Number,
            imeAction = ImeAction.Done,
            autoCorrect = false,
        ),
        keyboardActions = KeyboardActions(
            onDone = { triggerSubmit() }
        ),
        decorationBox = { decorationBox ->
            Box(
                modifier = Modifier
                    .padding(PaddingValues(12.dp, 7.dp))
                    .fillMaxWidth()
            ) {
                if (value.isBlank()) {
                    Text(
                        text = placeholderText,
                        color = placeholderTextColor,
                        textAlign = TextAlign.End,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
                decorationBox()
            }
        },
        cursorBrush = SolidColor(MullvadBlue)
    )
}

package net.mullvad.mullvadvpn.compose.component

import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.sp

internal val DEFAULT_TEXT_STEP = 1.sp

@Composable
fun CapsText(
    text: String,
    modifier: Modifier = Modifier,
    color: Color = Color.Unspecified,
    fontSize: TextUnit = TextUnit.Unspecified,
    fontStyle: androidx.compose.ui.text.font.FontStyle? = null,
    fontWeight: FontWeight? = null,
    fontFamily: FontFamily? = null,
    letterSpacing: TextUnit = TextUnit.Unspecified,
    textDecoration: TextDecoration? = null,
    textAlign: TextAlign? = null,
    lineHeight: TextUnit = TextUnit.Unspecified,
    overflow: TextOverflow = TextOverflow.Clip,
    softWrap: Boolean = true,
    maxLines: Int = Int.MAX_VALUE,
    onTextLayout: (TextLayoutResult) -> Unit = {},
    style: TextStyle = LocalTextStyle.current
) {
    Text(
        text = text.uppercase(),
        modifier = modifier,
        color = color,
        fontSize = fontSize,
        fontStyle = fontStyle,
        fontWeight = fontWeight,
        fontFamily = fontFamily,
        letterSpacing = letterSpacing,
        textDecoration = textDecoration,
        textAlign = textAlign,
        lineHeight = lineHeight,
        overflow = overflow,
        softWrap = softWrap,
        maxLines = maxLines,
        onTextLayout = onTextLayout,
        style = style,
    )
}

@Composable
fun AutoResizeText(
    text: String,
    minTextSize: TextUnit,
    maxTextSize: TextUnit,
    modifier: Modifier = Modifier,
    textSizeStep: TextUnit = DEFAULT_TEXT_STEP,
    style: TextStyle = LocalTextStyle.current,
    maxLines: Int = Int.MAX_VALUE,
) {
    var adjustedFontSize by remember { mutableFloatStateOf(maxTextSize.value) }
    var isReadyToDraw by remember { mutableStateOf(false) }

    Text(
        text = text,
        maxLines = maxLines,
        style = style,
        color = MaterialTheme.colorScheme.onBackground,
        fontSize = adjustedFontSize.sp,
        onTextLayout = {
            if (it.didOverflowHeight && isReadyToDraw.not()) {
                val nextFontSizeValue = adjustedFontSize - textSizeStep.value
                if (nextFontSizeValue <= minTextSize.value) {
                    adjustedFontSize = minTextSize.value
                    isReadyToDraw = true
                } else {
                    adjustedFontSize = nextFontSizeValue
                }
            } else {
                isReadyToDraw = true
            }
        },
        modifier = modifier.drawWithContent { if (isReadyToDraw) drawContent() }
    )
}

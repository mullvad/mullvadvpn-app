package net.mullvad.mullvadvpn.compose.button

import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.painter.Painter
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.lib.theme.Dimens

internal const val PRESS_EFFECT_TIME_SPAN: Long = 1000

@Composable
fun AnimatedIconButton(
    defaultIcon: Painter,
    secondaryIcon: Painter,
    modifier: Modifier = Modifier,
    pressEffectDuration: Long = PRESS_EFFECT_TIME_SPAN,
    defaultIconColorFilter: ColorFilter? = null,
    secondaryIconColorFilter: ColorFilter? = null,
    contentDescription: String,
    isToggleButton: Boolean = false,
    onClick: () -> Unit
) {
    var state by remember { mutableStateOf(ButtonState.IDLE) }
    if (state == ButtonState.PRESSED) {
        LaunchedEffect(Unit) {
            delay(pressEffectDuration)
            state = ButtonState.IDLE
        }
    }
    Box(
        modifier =
            modifier
                .clickable {
                    when (state) {
                        ButtonState.IDLE -> {
                            state = if (isToggleButton) ButtonState.TOGGLED else ButtonState.PRESSED
                            onClick()
                        }
                        ButtonState.TOGGLED -> {
                            state = ButtonState.IDLE
                            onClick()
                        }
                        ButtonState.PRESSED -> {}
                    }
                }
                .padding(all = Dimens.smallPadding)
    ) {
        AnimatedContent(targetState = state, label = contentDescription) { targetState ->
            val iconPainter: Painter
            val colorFilter: ColorFilter?
            val imageModifier: Modifier
            when (targetState) {
                ButtonState.IDLE -> {
                    iconPainter = defaultIcon
                    colorFilter = defaultIconColorFilter
                    imageModifier = modifier
                }
                ButtonState.TOGGLED -> {
                    iconPainter = secondaryIcon
                    colorFilter = secondaryIconColorFilter
                    imageModifier = modifier
                }
                ButtonState.PRESSED -> {
                    iconPainter = secondaryIcon
                    colorFilter = secondaryIconColorFilter
                    imageModifier = modifier
                }
            }

            Image(
                painter = iconPainter,
                colorFilter = colorFilter,
                contentDescription = contentDescription,
                modifier = imageModifier
            )
        }
    }
}

enum class ButtonState {
    IDLE,
    TOGGLED,
    PRESSED
}

package net.mullvad.mullvadvpn.compose.button

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.ExperimentalAnimationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.res.stringResource
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.R

internal const val PRESS_EFFECT_TIME_SPAN: Long = 1000

@OptIn(ExperimentalAnimationApi::class)
@Composable
fun AnimatedIconButton(
    defaultIcon: Painter,
    secondaryIcon: Painter,
    modifier: Modifier = Modifier,
    pressEffectDuration: Long = PRESS_EFFECT_TIME_SPAN,
    defaultIconColorFilter: ColorFilter? = null,
    secondaryIconColorFilter: ColorFilter? = null,
    isToggleButton: Boolean = false,
    onClick: () -> Unit
) {
    var state by remember { mutableStateOf(ButtonState.IDLE) }
    AnimatedContent(targetState = state) { targetState ->
        when (targetState) {
            ButtonState.IDLE -> {
                Image(
                    painter = defaultIcon,
                    colorFilter = defaultIconColorFilter,
                    contentDescription = "",
                    modifier =
                        modifier.clickable {
                            onClick()
                            state = if (isToggleButton) ButtonState.TOGGLED else ButtonState.PRESSED
                        }
                )
            }
            ButtonState.TOGGLED -> {
                Image(
                    painter = secondaryIcon,
                    colorFilter = secondaryIconColorFilter,
                    contentDescription = stringResource(id = R.string.copy_account_number),
                    modifier =
                        modifier.clickable {
                            onClick()
                            state = ButtonState.IDLE
                        }
                )
            }
            ButtonState.PRESSED -> {
                LaunchedEffect(Unit) {
                    delay(pressEffectDuration)
                    state = ButtonState.IDLE
                }
                Image(
                    painter = secondaryIcon,
                    colorFilter = secondaryIconColorFilter,
                    contentDescription = stringResource(id = R.string.copy_account_number),
                    modifier = modifier
                )
            }
        }
    }
}

enum class ButtonState {
    IDLE,
    TOGGLED,
    PRESSED
}

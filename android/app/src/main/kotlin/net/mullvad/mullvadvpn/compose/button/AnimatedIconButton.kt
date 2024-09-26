package net.mullvad.mullvadvpn.compose.button

import androidx.compose.animation.AnimatedContent
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import kotlinx.coroutines.delay

internal const val PRESS_EFFECT_TIME_SPAN: Long = 1000

@Composable
fun AnimatedIconButton(
    defaultIcon: ImageVector,
    secondaryIcon: ImageVector,
    pressEffectDuration: Long = PRESS_EFFECT_TIME_SPAN,
    defaultIconTint: Color,
    secondaryIconTint: Color,
    contentDescription: String,
    isToggleButton: Boolean = false,
    onClick: () -> Unit,
) {
    var state by remember { mutableStateOf(ButtonState.IDLE) }
    if (state == ButtonState.PRESSED) {
        LaunchedEffect(Unit) {
            delay(pressEffectDuration)
            state = ButtonState.IDLE
        }
    }

    IconButton(
        onClick = {
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
    ) {
        AnimatedContent(targetState = state, label = contentDescription) { targetState ->
            val imageVector: ImageVector
            val tint: Color
            when (targetState) {
                ButtonState.IDLE -> {
                    imageVector = defaultIcon
                    tint = defaultIconTint
                }
                ButtonState.TOGGLED -> {
                    imageVector = secondaryIcon
                    tint = secondaryIconTint
                }
                ButtonState.PRESSED -> {
                    imageVector = secondaryIcon
                    tint = secondaryIconTint
                }
            }

            Icon(imageVector = imageVector, contentDescription = contentDescription, tint = tint)
        }
    }
}

enum class ButtonState {
    IDLE,
    TOGGLED,
    PRESSED,
}

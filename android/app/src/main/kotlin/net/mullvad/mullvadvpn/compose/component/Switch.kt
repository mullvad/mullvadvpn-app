package net.mullvad.mullvadvpn.compose.component

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.SwitchColors
import androidx.compose.material.SwitchDefaults
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R

@Preview
@Composable
fun SwitchPreview() {

    CellSwitch(
        checked = false,
    )
}

@Composable
fun CellSwitch(
    checked: Boolean,
    onCheckedChange: ((Boolean) -> Unit)? = null,
    modifier: Modifier = Modifier,
//    enabled: Boolean = true,
//    interactionSource: MutableInteractionSource = remember { MutableInteractionSource() },
    colors: SwitchColors = SwitchDefaults.colors(),
    scale: Float = 1f,
    thumbCheckedTrackColor: Color = colorResource(id = R.color.green),
    thumbUncheckedTrackColor: Color = colorResource(id = R.color.red),
    thumbColor: Color = colorResource(id = R.color.white),
) {
    var gapBetweenThumbAndTrackEdge: Dp = 2.dp

    var width: Dp = 46.dp
    var height: Dp = 28.dp
    val switchON = remember {
        mutableStateOf(checked)
    }

    val thumbRadius = 11.dp

    // To move the thumb, we need to calculate the position (along x axis)
    val animatePosition = animateFloatAsState(
        targetValue = if (switchON.value)
            with(LocalDensity.current) {
                (width - thumbRadius - gapBetweenThumbAndTrackEdge - 1.dp).toPx()
            }
        else
            with(LocalDensity.current) { (thumbRadius + gapBetweenThumbAndTrackEdge + 1.dp).toPx() }
    )

    Canvas(
        modifier = modifier
            .padding(1.dp)
            .size(width = width, height = height)
            .scale(scale = scale)
            .pointerInput(Unit) {
                detectTapGestures(
                    onTap = {
                        // This is called when the user taps on the canvas
                        switchON.value = !switchON.value
                        onCheckedChange?.let { it(switchON.value) }
                    }
                )
            }
    ) {
        // Track
        drawRoundRect(
            color = thumbColor,
            cornerRadius = CornerRadius(x = 15.dp.toPx(), y = 15.dp.toPx()),
            style = Stroke(
                width = 2.dp.toPx(),
                miter = 6.dp.toPx(),
                cap = StrokeCap.Square,
            ),
        )

        // Thumb
        drawCircle(
            color = if (switchON.value) thumbCheckedTrackColor else thumbUncheckedTrackColor,
            radius = thumbRadius.toPx(),
            center = Offset(
                x = animatePosition.value,
                y = size.height / 2
            )
        )
    }

    Spacer(modifier = Modifier.height(18.dp))

    Text(text = if (switchON.value) "" else "")
}

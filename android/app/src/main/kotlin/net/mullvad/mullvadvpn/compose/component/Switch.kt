package net.mullvad.mullvadvpn.compose.component

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.compose.theme.MullvadGreen
import net.mullvad.mullvadvpn.compose.theme.MullvadRed
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite

@Preview
@Composable
private fun PreviewSwitch() {
    CellSwitch(isChecked = false, onCheckedChange = null)
}

@Composable
fun CellSwitch(
    isChecked: Boolean,
    onCheckedChange: ((Boolean) -> Unit)?,
    modifier: Modifier = Modifier,
    scale: Float = 1f,
    thumbCheckedTrackColor: Color = MullvadGreen,
    thumbUncheckedTrackColor: Color = MullvadRed,
    thumbColor: Color = MullvadWhite
) {
    val gapBetweenThumbAndTrackEdge: Dp = 2.dp
    val width: Dp = 46.dp
    val height: Dp = 28.dp
    val thumbRadius = 11.dp

    // To move the thumb, we need to calculate the position (along x axis)
    val animatePosition =
        animateFloatAsState(
            targetValue =
                if (isChecked)
                    with(LocalDensity.current) {
                        (width - thumbRadius - gapBetweenThumbAndTrackEdge - 1.dp).toPx()
                    }
                else
                    with(LocalDensity.current) {
                        (thumbRadius + gapBetweenThumbAndTrackEdge + 1.dp).toPx()
                    }
        )

    Canvas(
        modifier =
            modifier
                .padding(1.dp)
                .size(width = width, height = height)
                .scale(scale = scale)
                .pointerInput(Unit) {
                    if (onCheckedChange != null) {
                        detectTapGestures(onTap = { onCheckedChange(!isChecked) })
                    }
                }
    ) {
        // Track
        drawRoundRect(
            color = thumbColor,
            cornerRadius = CornerRadius(x = 15.dp.toPx(), y = 15.dp.toPx()),
            style =
                Stroke(
                    width = 2.dp.toPx(),
                    miter = 6.dp.toPx(),
                    cap = StrokeCap.Square,
                ),
        )

        // Thumb
        drawCircle(
            color = if (isChecked) thumbCheckedTrackColor else thumbUncheckedTrackColor,
            radius = thumbRadius.toPx(),
            center = Offset(x = animatePosition.value, y = size.height / 2)
        )
    }

    Spacer(modifier = Modifier.height(18.dp))
}

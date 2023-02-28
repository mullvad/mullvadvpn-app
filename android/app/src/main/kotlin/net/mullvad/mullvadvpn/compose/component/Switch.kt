package net.mullvad.mullvadvpn.compose.component

import android.util.Log
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.Text
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
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R

@Preview
@Composable
private fun PreviewSwitch() {
    CellSwitch(
        checked = false,
        onCheckedChange = {}
    )
}

private var isChecked: Boolean = false
var getChecked: Boolean
    get() = isChecked
    set(value) { isChecked = value }

@Composable
fun CellSwitch(
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
    scale: Float = 1f,
    thumbCheckedTrackColor: Color = colorResource(id = R.color.green),
    thumbUncheckedTrackColor: Color = colorResource(id = R.color.red),
    thumbColor: Color = colorResource(id = R.color.white),
) {
    val gapBetweenThumbAndTrackEdge: Dp = 2.dp
    val width: Dp = 46.dp
    val height: Dp = 28.dp
    val thumbRadius = 11.dp
    getChecked = checked

    // To move the thumb, we need to calculate the position (along x axis)
    val animatePosition = animateFloatAsState(
        targetValue = if (checked)
            with(LocalDensity.current) {
                (width - thumbRadius - gapBetweenThumbAndTrackEdge - 1.dp).toPx()
            }
        else
            with(LocalDensity.current) { (thumbRadius + gapBetweenThumbAndTrackEdge + 1.dp).toPx() }
    )

    Log.d("mullvad", "AAA compose isChecked: $checked")

    Canvas(
        modifier = modifier
            .padding(1.dp)
            .size(width = width, height = height)
            .scale(scale = scale)
            .pointerInput(Unit) {
                detectTapGestures(
                    onTap = {
                        // Investigate behaviour of Canvas onTap function later, it keeps initial state
                        onCheckedChange(!getChecked)
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
            color = if (checked) thumbCheckedTrackColor else thumbUncheckedTrackColor,
            radius = thumbRadius.toPx(),
            center = Offset(
                x = animatePosition.value,
                y = size.height / 2
            )
        )
    }

    Spacer(modifier = Modifier.height(18.dp))

    Text(text = if (checked) "" else "")
}

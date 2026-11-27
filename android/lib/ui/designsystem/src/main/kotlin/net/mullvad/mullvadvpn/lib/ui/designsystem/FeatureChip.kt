package net.mullvad.mullvadvpn.lib.ui.designsystem

import android.os.Parcelable
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.animateColor
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.Transition
import androidx.compose.animation.core.animateDp
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.core.updateTransition
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.minimumInteractiveComponentSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.State
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.center
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Matrix
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathMeasure
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import kotlin.math.atan2
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.ui.designsystem.preview.PreviewColumn
import net.mullvad.mullvadvpn.lib.ui.icon.MultihopWhenNeeded
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.shape.chipShape
import net.mullvad.mullvadvpn.lib.ui.util.applyIf

@Preview
@Composable
private fun PreviewMullvadFeatureChip() {
    PreviewColumn(Modifier.background(Color.Black).padding(16.dp)) {
        MullvadFeatureChip(text = "DAITA", state = ChipState.Idle, onClick = {})
        MullvadFeatureChip(text = "Local Network Sharing", state = ChipState.Idle, onClick = {})
        MullvadFeatureChip(
            text = "Multihop",
            state = ChipState.Idle,
            icon = MultihopWhenNeeded,
            onClick = {},
        )
        var state by remember { mutableStateOf(false) }

        MullvadSwitch(
            checked = state,
            onCheckedChange = { state = !state },
        )

        MullvadFeatureChip(
            text = "Anti-censorship",
            state = if (state) ChipState.Loading else ChipState.Idle,
            onClick = {},
        )
        MullvadFeatureChip(text = "QUIC", state = ChipState.Idle, onClick = {})
    }
}

@Composable
@Suppress("LongMethod")
fun MullvadFeatureChip(
    text: String,
    state: ChipState,
    modifier: Modifier = Modifier,
    icon: ImageVector? = null,
    onClick: () -> Unit,
    containerColor: Color = MaterialTheme.colorScheme.surfaceContainerLowest,
    borderColor: Color = MaterialTheme.colorScheme.primary,
    highlightBorderColor: Color = MaterialTheme.colorScheme.onPrimary,
    labelColor: Color = MaterialTheme.colorScheme.onPrimary,
    iconColor: Color = MaterialTheme.colorScheme.onPrimary,
) {
    val chipShape = MaterialTheme.shapes.chipShape
    val transition = updateTransition(targetState = state, label = "chipStateTransition")

    val borderWidth = transition.animateBorderWidth()
    val animatedBorderColor = transition.animateBorderColor(highlightBorderColor, borderColor)

    // This is to handle loading state animations better.
    // Without this the chip itself will add the minimum interactive size
    // causing issues with aligning the border properly.
    Box(
        modifier = modifier.minimumInteractiveComponentSize(),
        contentAlignment = Alignment.Center,
    ) {
        CompositionLocalProvider(
            LocalMinimumInteractiveComponentSize provides Dp.Unspecified,
        ) {
            FilterChip(
                modifier =
                    Modifier.applyIf(state == ChipState.Loading) {
                        val connectingProgress =
                            rememberInfiniteTransition(label = "connectingBorder")
                                .animateFloat(
                                    initialValue = 0f,
                                    targetValue = 1f,
                                    animationSpec =
                                        infiniteRepeatable(
                                            animation =
                                                tween(
                                                    LOADING_BORDER_LOOP_DURATION_MS,
                                                    easing = LinearEasing,
                                                ),
                                        ),
                                    label = "connectingBorderFloat",
                                )
                        connectingBorder(
                            shape = chipShape,
                            sweepColors =
                                listOf(
                                    borderColor.copy(0f),
                                    borderColor.copy(0.3f),
                                    borderColor,
                                ),
                            borderWidth = LOADING_BORDER_WIDTH,
                            progress = { connectingProgress.value },
                        )
                    },
                shape = chipShape,
                colors =
                    FilterChipDefaults.filterChipColors(
                        containerColor = containerColor,
                        disabledLabelColor = labelColor,
                        labelColor = labelColor,
                        iconColor = iconColor,
                    ),
                border =
                    FilterChipDefaults.filterChipBorder(
                        borderColor = animatedBorderColor.value,
                        enabled = true,
                        selected = false,
                        borderWidth = borderWidth.value,
                    ),
                selected = false,
                onClick = onClick,
                label = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        icon?.let {
                            Icon(
                                modifier = Modifier.size(Dimens.featureIndicatorIconSize),
                                imageVector = it,
                                tint = MaterialTheme.colorScheme.onSurface,
                                contentDescription = null,
                            )
                            Spacer(Modifier.width(Dimens.smallPadding))
                        }
                        AnimatedContent(
                            targetState = text,
                            transitionSpec = {
                                fadeIn() togetherWith fadeOut() using SizeTransform()
                            },
                            label = "featureChipText",
                        ) {
                            Text(
                                text = it,
                                style = MaterialTheme.typography.bodyMedium,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                },
            )
        }
    }
}

@Composable
private fun Transition<ChipState>.animateBorderWidth(): State<Dp> =
    animateDp(
        transitionSpec = {
            when (targetState) {
                ChipState.Idle ->
                    keyframes {
                        durationMillis = HIGHLIGHT_DURATION_MS
                        LOADING_BORDER_WIDTH at HIGHLIGHT_KEYFRAME_HOLD_MS
                    }
                ChipState.Loading -> tween()
            }
        },
    ) {
        when (it) {
            ChipState.Idle -> IDLE_BORDER_WIDTH
            ChipState.Loading -> LOADING_BORDER_WIDTH
        }
    }

@Composable
private fun Transition<ChipState>.animateBorderColor(
    highlightBorderColor: Color,
    borderColor: Color,
): State<Color> =
    animateColor(
        transitionSpec = {
            when (targetState) {
                ChipState.Idle ->
                    keyframes {
                        durationMillis = HIGHLIGHT_DURATION_MS
                        highlightBorderColor at HIGHLIGHT_KEYFRAME_FADE_IN_MS
                        highlightBorderColor at HIGHLIGHT_KEYFRAME_HOLD_MS
                    }
                ChipState.Loading -> tween()
            }
        },
    ) {
        when (it) {
            ChipState.Idle -> borderColor
            ChipState.Loading -> Color.Transparent
        }
    }

@Composable
private fun Modifier.connectingBorder(
    shape: Shape,
    sweepColors: List<Color>,
    borderWidth: Dp,
    progress: () -> Float,
): Modifier = drawWithCache {
    // We double the path to keep the right radius, the outer half will be clipped away.
    val stroke = Stroke(borderWidth.toPx() * 2)

    val path = shape.createOutline(size, layoutDirection, this).toPath()
    val measure = PathMeasure().apply { setPath(path, true) }
    val length = measure.length

    val segments = Path()

    val brush = Brush.sweepGradient(colors = sweepColors, center = size.center)
    val counterRotation = Matrix()

    onDrawWithContent {
        drawContent()

        val head = (1f - progress()) * length

        clipPath(path) {
            // Since we have a sweepingBrush and want a nice Stroke.But, we need to create a gap
            val end = head + length * PATH_DRAW_FACTOR

            segments.rewind()
            if (end <= length) {
                measure.getSegment(head, end, segments)
            } else {
                // If we are wrapping around the start we need to create two segments
                measure.getSegment(head, length, segments)
                measure.getSegment(0f, end - length, segments, false)
            }

            // The angle won't have a consistent speed, it depends on the position of the head.
            val headPosition = measure.getPosition(head)
            val angle = angleToPoint(center, headPosition) + BRUSH_LEAD_DEGREE

            // Rotating the segment by -angle around the center cancels the canvas
            // rotation below, so only the gradient visibly rotates.
            counterRotation.resetToPivotedTransform(center.x, center.y, rotationZ = -angle)
            segments.transform(counterRotation)
            clipPath(path) {
                rotate(degrees = angle, pivot = center) {
                    drawPath(segments, brush = brush, style = stroke)
                }
            }
        }
    }
}

private fun angleToPoint(center: Offset, point: Offset): Float {
    val dx = point.x - center.x
    val dy = point.y - center.y
    val degrees = Math.toDegrees(atan2(dy, dx).toDouble()).toFloat()
    return (degrees + 360f) % 360f
}

private fun Outline.toPath() =
    when (this) {
        is Outline.Generic -> path
        is Outline.Rounded -> Path().apply { addRoundRect(roundRect) }
        is Outline.Rectangle -> Path().apply { addRect(rect) }
    }

@Composable
fun MullvadMoreChip(
    onClick: () -> Unit,
    containerColor: Color = MaterialTheme.colorScheme.background,
    borderColor: Color = Color.Transparent,
    labelColor: Color = MaterialTheme.colorScheme.onPrimary,
    iconColor: Color = MaterialTheme.colorScheme.onPrimary,
    text: String,
) {
    FilterChip(
        onClick = onClick,
        shape = MaterialTheme.shapes.chipShape,
        colors =
            FilterChipDefaults.filterChipColors(
                containerColor = containerColor,
                labelColor = labelColor,
                iconColor = iconColor,
            ),
        border =
            FilterChipDefaults.filterChipBorder(
                borderColor = borderColor,
                enabled = true,
                selected = false,
            ),
        selected = false,
        label = {
            Text(
                text = text,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        },
    )
}

@Parcelize
sealed interface ChipState : Parcelable {
    data object Idle : ChipState

    data object Loading : ChipState
}

private val IDLE_BORDER_WIDTH = 1.dp
private val LOADING_BORDER_WIDTH = 2.dp

private const val BRUSH_LEAD_DEGREE = 5f
private const val PATH_DRAW_FACTOR = 0.9f
private const val LOADING_BORDER_LOOP_DURATION_MS = 2000 // 2 seconds

private const val HIGHLIGHT_KEYFRAME_FADE_IN_MS = 100
private const val HIGHLIGHT_KEYFRAME_HOLD_MS = 700
private const val HIGHLIGHT_DURATION_MS = 1300

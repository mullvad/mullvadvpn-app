package net.mullvad.mullvadvpn.lib.ui.designsystem

import android.os.Parcelable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathMeasure
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.ui.designsystem.preview.PreviewColumn
import net.mullvad.mullvadvpn.lib.ui.icon.MultihopWhenNeeded
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.shape.chipShape
import net.mullvad.mullvadvpn.lib.ui.util.applyIf

@Preview
@Composable
private fun PreviewMullvadFeatureChip() {
    PreviewColumn {
        MullvadFeatureChip(text = "DAITA", state = State.Idle, onClick = {})
        MullvadFeatureChip(text = "Local Network Sharing", state = State.Idle, onClick = {})
        MullvadFeatureChip(
            text = "Multihop",
            state = State.Idle,
            icon = MultihopWhenNeeded,
            onClick = {},
        )
        MullvadFeatureChip(text = "Anti-censorship", state = State.Connecting, onClick = {})
        MullvadFeatureChip(text = "QUIC", state = State.Connected, onClick = {})
    }
}

@Composable
fun MullvadFeatureChip(
    text: String,
    state: State,
    modifier: Modifier = Modifier,
    icon: ImageVector? = null,
    onClick: () -> Unit,
    containerColor: Color = MaterialTheme.colorScheme.surfaceContainerLowest,
    borderColor: Color = MaterialTheme.colorScheme.primary,
    selectedBorderColor: Color = MaterialTheme.colorScheme.onPrimary,
    labelColor: Color = MaterialTheme.colorScheme.onPrimary,
    iconColor: Color = MaterialTheme.colorScheme.onPrimary,
) {
    val selected = state == State.Connected
    val chipShape = MaterialTheme.shapes.chipShape
    val rotation =
        if (state == State.Connecting) {
            rememberInfiniteTransition(label = "connectingBorder")
                .animateFloat(
                    initialValue = 0f,
                    targetValue = 1f,
                    animationSpec =
                        infiniteRepeatable(
                            animation = tween(CONNECTING_BORDER_DURATION_MS, easing = LinearEasing),
                        ),
                    label = "connectingBorderAngle",
                )
        } else {
            null
        }
    FilterChip(
        modifier =
            modifier.applyIf(rotation != null) {
                connectingBorder(
                    shape = chipShape,
                    color = selectedBorderColor,
                    progress = { rotation!!.value },
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
                borderColor = borderColor,
                selectedBorderColor = selectedBorderColor,
                selectedBorderWidth = SELECTED_BORDER_WIDTH,
                enabled = true,
                selected = selected,
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
                Text(
                    text = text,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        },
    )
}

private fun Modifier.connectingBorder(
    shape: Shape,
    color: Color,
    progress: () -> Float,
): Modifier = drawWithContent {
    drawContent()
    val strokeWidth = SELECTED_BORDER_WIDTH.toPx()
    val inset = strokeWidth / 2
    val chipHeight = minOf(FilterChipDefaults.Height.toPx(), size.height)
    val verticalOffset = (size.height - chipHeight) / 2
    val outline =
        shape.createOutline(
            size = Size(size.width - strokeWidth, chipHeight - strokeWidth),
            layoutDirection = layoutDirection,
            density = this,
        )
    val path =
        when (outline) {
            is Outline.Generic -> outline.path
            is Outline.Rounded -> Path().apply { addRoundRect(outline.roundRect) }
            is Outline.Rectangle -> Path().apply { addRect(outline.rect) }
        }
    val measure = PathMeasure().apply { setPath(path, true) }
    val length = measure.length
    val head = progress() * length
    val sliceLength = length / CONNECTING_ARC_SLICES
    val segment = Path()

    fun drawSlice(start: Float, end: Float, alpha: Float) {
        segment.rewind()
        measure.getSegment(start, end, segment, true)
        drawPath(segment, color.copy(alpha = alpha), style = Stroke(strokeWidth))
    }

    translate(inset, verticalOffset + inset) {
        for (i in 0 until CONNECTING_ARC_SLICES) {
            val end = head - sliceLength * i
            val start = end - sliceLength
            val alpha = 1f - i.toFloat() / CONNECTING_ARC_SLICES
            if (start >= 0f) {
                drawSlice(start, end, alpha)
            } else if (end <= 0f) {
                drawSlice(start + length, end + length, alpha)
            } else {
                drawSlice(0f, end, alpha)
                drawSlice(start + length, length, alpha)
            }
        }
    }
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
sealed interface State : Parcelable {
    data object Idle : State

    data object Connecting : State

    data object Connected : State
}

private val SELECTED_BORDER_WIDTH = 2.dp
private const val CONNECTING_BORDER_DURATION_MS = 1500
private const val CONNECTING_ARC_SLICES = 32

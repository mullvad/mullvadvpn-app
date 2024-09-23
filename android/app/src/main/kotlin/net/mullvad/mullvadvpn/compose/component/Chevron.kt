package net.mullvad.mullvadvpn.compose.component

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.TweenSpec
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R

@Composable
@Preview
private fun PreviewChevron() {
    Column {
        Chevron(color = MaterialTheme.colorScheme.onPrimary, isExpanded = false)
        Chevron(color = MaterialTheme.colorScheme.onPrimary, isExpanded = true)
    }
}

@Composable
@Preview
private fun PreviewChevronRight() {
    Column {
        ChevronLeft(tint = MaterialTheme.colorScheme.onPrimary)
        ChevronRight(tint = MaterialTheme.colorScheme.onPrimary)
    }
}

@Composable
fun Chevron(modifier: Modifier = Modifier, color: Color, isExpanded: Boolean) {

    val degree = remember(isExpanded) { if (isExpanded) UP_ROTATION else DOWN_ROTATION }
    val animatedRotation =
        animateFloatAsState(
            targetValue = degree,
            label = "",
            animationSpec = TweenSpec(100, easing = LinearEasing),
        )

    Icon(
        painterResource(id = R.drawable.icon_chevron),
        contentDescription = null,
        tint = color,
        modifier = modifier.rotate(animatedRotation.value),
    )
}

@Composable
fun ChevronLeft(modifier: Modifier = Modifier, tint: Color, contentDescription: String? = null) {
    Icon(
        painterResource(id = R.drawable.icon_chevron),
        contentDescription = contentDescription,
        tint = tint,
        modifier = modifier.rotate(LEFT_ROTATION),
    )
}

@Composable
fun ChevronRight(modifier: Modifier = Modifier, tint: Color, contentDescription: String? = null) {
    Icon(
        painterResource(id = R.drawable.icon_chevron),
        contentDescription = contentDescription,
        tint = tint,
        modifier = modifier.rotate(RIGHT_ROTATION),
    )
}

@Composable
fun ExpandChevronIconButton(
    modifier: Modifier = Modifier,
    color: Color,
    onExpand: (Boolean) -> Unit,
    isExpanded: Boolean,
) {
    IconButton(modifier = modifier, onClick = { onExpand(!isExpanded) }) {
        Chevron(isExpanded = isExpanded, color = color)
    }
}

private const val RIGHT_ROTATION = -90f
private const val LEFT_ROTATION = 90f
private const val DOWN_ROTATION = 0f
private const val UP_ROTATION = 180f

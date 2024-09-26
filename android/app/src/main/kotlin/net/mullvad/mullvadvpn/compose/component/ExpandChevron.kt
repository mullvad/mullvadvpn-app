package net.mullvad.mullvadvpn.compose.component

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.TweenSpec
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.layout.Column
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview

@Composable
@Preview
private fun PreviewChevron() {
    Column {
        ExpandChevron(color = MaterialTheme.colorScheme.onPrimary, isExpanded = false)
        ExpandChevron(color = MaterialTheme.colorScheme.onPrimary, isExpanded = true)
    }
}

@Composable
fun ExpandChevron(modifier: Modifier = Modifier, color: Color, isExpanded: Boolean) {

    val degree = remember(isExpanded) { if (isExpanded) UP_ROTATION else DOWN_ROTATION }
    val animatedRotation =
        animateFloatAsState(
            targetValue = degree,
            label = "",
            animationSpec = TweenSpec(100, easing = LinearEasing),
        )

    Icon(
        imageVector = Icons.Default.KeyboardArrowDown,
        contentDescription = null,
        tint = color,
        modifier = modifier.rotate(animatedRotation.value),
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
        ExpandChevron(isExpanded = isExpanded, color = color)
    }
}

private const val DOWN_ROTATION = 0f
private const val UP_ROTATION = 180f

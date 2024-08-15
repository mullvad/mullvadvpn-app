package net.mullvad.mullvadvpn.compose.component

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.TweenSpec
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import net.mullvad.mullvadvpn.R

@Composable
fun Chevron(
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.onBackground,
    isExpanded: Boolean
) {

    val degree = remember(isExpanded) { if (isExpanded) 270f else 90f }
    val animatedRotation =
        animateFloatAsState(
            targetValue = degree,
            label = "",
            animationSpec = TweenSpec(100, easing = LinearEasing)
        )

    Icon(
        painterResource(id = R.drawable.icon_chevron),
        contentDescription = null,
        tint = color,
        modifier = modifier.rotate(animatedRotation.value),
    )
}

@Composable
fun ChevronButton(
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.onBackground,
    onExpand: (Boolean) -> Unit,
    isExpanded: Boolean
) {
    IconButton(modifier = modifier, onClick = { onExpand(!isExpanded) }) {
        Chevron(isExpanded = isExpanded, color = color)
    }
}

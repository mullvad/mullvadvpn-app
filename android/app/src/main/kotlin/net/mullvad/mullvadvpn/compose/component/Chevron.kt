package net.mullvad.mullvadvpn.compose.component

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.res.painterResource
import net.mullvad.mullvadvpn.R

@Composable
fun ChevronView(
    modifier: Modifier = Modifier,
    colorFilter: ColorFilter? = null,
    isExpanded: Boolean
) {
    val resourceId = R.drawable.icon_chevron
    val rotation = remember { Animatable(90f + if (isExpanded) 180f else 0f) }

    LaunchedEffect(isExpanded) {
        rotation.animateTo(
            targetValue = 90f + if (isExpanded) 180f else 0f,
            animationSpec = tween(100, easing = LinearEasing)
        )
    }

    Image(
        painterResource(id = resourceId),
        contentDescription = null,
        colorFilter = colorFilter,
        modifier = modifier.rotate(rotation.value),
    )
}

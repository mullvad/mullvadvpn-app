package net.mullvad.mullvadvpn.compose.component

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R

@Composable
fun ChevronView(isExpanded: Boolean) {
    val resourceId = R.drawable.icon_chevron
    val rotation = remember { Animatable(90f) }

    LaunchedEffect(isExpanded) {
        rotation.animateTo(
            targetValue = 90f + if (isExpanded) 0f else 180f,
            animationSpec = tween(100, easing = LinearEasing)
        )
    }

    Image(
        painterResource(id = resourceId),
        contentDescription = null,
        modifier = Modifier.size(30.dp).rotate(rotation.value),
    )
}

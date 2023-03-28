package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite

@Preview
@Composable
private fun PreviewExpandableComposeCell() {
    ExpandableComposeCell(
        title = "Expandable row title",
        expandState = true,
        onCellClicked = {},
        onInfoClicked = {},
    )
}

@Composable
fun ExpandableComposeCell(
    title: String,
    expandState: Boolean,
    titleAlpha: Float = 1f,
    onCellClicked: (Boolean) -> Unit = {},
    onInfoClicked: (() -> Unit)? = null
) {
    val titleModifier = Modifier.alpha(titleAlpha)
    val bodyViewModifier = Modifier

    BaseCell(
        title = { SwitchCellTitle(title = title, modifier = titleModifier) },
        bodyView = {
            RightCellView(
                isExpanded = expandState,
                modifier = bodyViewModifier,
                onInfoClicked = onInfoClicked,
            )
        },
        onCellClicked = { onCellClicked(!expandState) },
    )
}

@Composable
private fun RightCellView(
    isExpanded: Boolean,
    modifier: Modifier,
    onInfoClicked: (() -> Unit)? = null
) {
    val horizontalPadding = dimensionResource(id = R.dimen.medium_padding)
    val verticalPadding = 13.dp
    Row(
        modifier = modifier
            .wrapContentWidth()
            .wrapContentHeight(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        if (onInfoClicked != null) {
            Icon(
                modifier = Modifier
                    .clickable { onInfoClicked() }
                    .padding(
                        start = horizontalPadding,
                        end = horizontalPadding,
                        top = verticalPadding,
                        bottom = verticalPadding,
                    )
                    .align(Alignment.CenterVertically),
                painter = painterResource(id = R.drawable.icon_info),
                contentDescription = stringResource(id = R.string.confirm_local_dns),
                tint = MullvadWhite,
            )
        }

        ChevronView(isExpanded)
    }
}

@Composable
fun ChevronView(
    isExpanded: Boolean
) {
    val resourceId = R.drawable.icon_chevron
    val rotation = remember { Animatable(90f) }
    rememberCoroutineScope().let {
        it.launch {
            rotation.animateTo(
                targetValue = 90f + if (isExpanded) 0f else 180f,
                animationSpec = tween(100, easing = LinearEasing),
            )
        }
    }

    Image(
        painterResource(id = resourceId),
        contentDescription = null,
        modifier = Modifier
            .size(30.dp)
            .rotate(rotation.value),
    )
}

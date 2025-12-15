package net.mullvad.mullvadvpn.lib.ui.component.listitem

import android.graphics.drawable.Drawable
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Android
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.google.accompanist.drawablepainter.rememberDrawablePainter
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.ui.component.preview.PreviewSpacedColumn
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Preview
@Composable
private fun PreviewSplitTunnelingListItem() {
    AppTheme {
        PreviewSpacedColumn {
            SplitTunnelingListItem(
                title = "Removable App",
                isEnabled = true,
                onCellClicked = {},
                isSelected = true,
                iconState = IconState.Loading,
            )
            SplitTunnelingListItem(
                title = "Addable App",
                isEnabled = true,
                onCellClicked = {},
                isSelected = false,
                iconState = IconState.Loading,
            )
            SplitTunnelingListItem(
                title = "Disabled App",
                isEnabled = false,
                onCellClicked = {},
                isSelected = false,
                iconState = IconState.NoIcon,
            )
        }
    }
}

@Composable
fun SplitTunnelingListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Middle,
    title: String,
    iconState: IconState,
    isEnabled: Boolean = true,
    isSelected: Boolean,
    backgroundAlpha: Float = 1f,
    onCellClicked: () -> Unit,
) {
    MullvadListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        isEnabled = isEnabled,
        onClick = onCellClicked,
        backgroundAlpha = backgroundAlpha,
        colors = ListItemDefaults.colors(),
        content = { Text(title) },
        leadingContent = { Icon(iconState = iconState, isEnabled) },
        trailingContent = {
            Icon(
                imageVector =
                    if (isSelected) {
                        Icons.Default.Remove
                    } else {
                        Icons.Default.Add
                    },
                contentDescription = null,
                tint =
                    MaterialTheme.colorScheme.onSurface.copy(
                        alpha = if (isEnabled) 1f else AlphaDisabled
                    ),
                modifier = Modifier.size(size = iconSize),
            )
        },
    )
}

@Composable
private fun BoxScope.Icon(iconState: IconState, isEnabled: Boolean) {
    when (iconState) {
        IconState.Loading ->
            Box(
                modifier =
                    Modifier.align(Alignment.Center)
                        .padding(end = Dimens.smallPadding)
                        .size(iconSize)
                        .clip(CircleShape)
                        .background(MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaDisabled))
            )
        is IconState.Icon -> {
            Image(
                painter = rememberDrawablePainter(drawable = iconState.drawable),
                contentDescription = null,
                alpha = if (isEnabled) 1f else AlphaDisabled,
                modifier =
                    Modifier.align(Alignment.Center)
                        .padding(end = Dimens.smallPadding)
                        .size(size = iconSize),
            )
        }
        IconState.NoIcon ->
            Image(
                imageVector = Icons.Default.Android,
                contentDescription = null,
                alpha = if (isEnabled) 1f else AlphaDisabled,
                modifier =
                    Modifier.align(Alignment.Center)
                        .padding(end = Dimens.smallPadding)
                        .size(size = iconSize),
                colorFilter = ColorFilter.tint(MaterialTheme.colorScheme.onSurface),
            )
    }
}

sealed class IconState {
    object Loading : IconState()

    data class Icon(val drawable: Drawable) : IconState()

    object NoIcon : IconState() // Icon not found or icon too large
}

private val iconSize: Dp = 24.dp

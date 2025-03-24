package net.mullvad.mullvadvpn.compose.cell

import android.graphics.drawable.Drawable
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.google.accompanist.drawablepainter.rememberDrawablePainter
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.compose.util.isBelowMaxSize
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemText

@Preview
@Composable
private fun PreviewTunnelingCell() {
    AppTheme {
        SpacedColumn(
            modifier = Modifier.background(color = MaterialTheme.colorScheme.surface).padding(20.dp)
        ) {
            SplitTunnelingCell(
                title = "Mullvad VPN",
                packageName = "",
                isSelected = false,
                enabled = true,
                onCellClicked = {},
                onResolveIcon = { null },
            )
            SplitTunnelingCell(
                title = "Mullvad VPN",
                packageName = "",
                isSelected = true,
                enabled = true,
                onCellClicked = {},
                onResolveIcon = { null },
            )
        }
    }
}

@Composable
fun SplitTunnelingCell(
    title: String,
    packageName: String?,
    isSelected: Boolean,
    enabled: Boolean,
    modifier: Modifier = Modifier,
    backgroundColor: Color = MaterialTheme.colorScheme.surfaceContainerHigh,
    onResolveIcon: (String) -> Drawable?,
    onCellClicked: () -> Unit,
) {
    var icon by remember(packageName) { mutableStateOf<IconState>(IconState.Loading) }
    LaunchedEffect(packageName) {
        launch(Dispatchers.IO) {
            val drawable = onResolveIcon(packageName ?: "")
            icon =
                if (drawable != null && drawable.isBelowMaxSize()) {
                    IconState.Icon(drawable = drawable)
                } else {
                    IconState.NoIcon
                }
        }
    }
    BaseCell(
        iconView = { Icon(iconState = icon) },
        headlineContent = {
            Text(
                text = title,
                style = MaterialTheme.typography.listItemText,
                color = MaterialTheme.colorScheme.onSurface,
                modifier =
                    Modifier.weight(1f)
                        .padding(horizontal = Dimens.mediumPadding)
                        .align(Alignment.CenterVertically),
            )
        },
        bodyView = {
            Icon(
                imageVector =
                    if (isSelected) {
                        Icons.Default.Remove
                    } else {
                        Icons.Default.Add
                    },
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.size(size = Dimens.addIconSize),
            )
        },
        onCellClicked = onCellClicked,
        background = backgroundColor,
        modifier = modifier,
        isRowEnabled = enabled,
    )
}

@Composable
private fun RowScope.Icon(iconState: IconState) {
    when (iconState) {
        IconState.Loading ->
            Box(
                modifier =
                    Modifier.align(Alignment.CenterVertically)
                        .size(Dimens.listIconSize)
                        .clip(CircleShape)
                        .background(MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaDisabled))
            )
        is IconState.Icon ->
            Image(
                painter = rememberDrawablePainter(drawable = iconState.drawable),
                contentDescription = null,
                modifier =
                    Modifier.align(Alignment.CenterVertically).size(size = Dimens.listIconSize),
            )
        IconState.NoIcon ->
            Image(
                painter = painterResource(id = R.drawable.ic_icons_missing),
                contentDescription = null,
                modifier =
                    Modifier.align(Alignment.CenterVertically).size(size = Dimens.listIconSize),
                colorFilter = ColorFilter.tint(MaterialTheme.colorScheme.onSurface),
            )
    }
}

private sealed class IconState {
    object Loading : IconState()

    data class Icon(val drawable: Drawable) : IconState()

    object NoIcon : IconState() // Icon not found or icon too large
}

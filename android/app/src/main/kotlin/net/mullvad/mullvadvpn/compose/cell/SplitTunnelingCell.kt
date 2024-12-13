package net.mullvad.mullvadvpn.compose.cell

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.painter.BitmapPainter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.compose.util.isBelowMaxBitmapSize
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
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
    onResolveIcon: (String) -> Bitmap?,
    onCellClicked: () -> Unit,
) {
    var icon by remember(packageName) { mutableStateOf<ImageBitmap?>(null) }
    LaunchedEffect(packageName) {
        launch(Dispatchers.IO) {
            val bitmap = onResolveIcon(packageName ?: "")
            if (bitmap != null && bitmap.isBelowMaxBitmapSize()) {
                icon = bitmap.asImageBitmap()
            }
        }
    }
    BaseCell(
        iconView = {
            Image(
                painter =
                    icon?.let { iconImage -> BitmapPainter(iconImage) }
                        ?: painterResource(id = R.drawable.ic_icons_missing),
                contentDescription = null,
                modifier =
                    Modifier.align(Alignment.CenterVertically).size(size = Dimens.listIconSize),
                colorFilter =
                    if (icon == null) {
                        ColorFilter.tint(MaterialTheme.colorScheme.onSurface)
                    } else {
                        null
                    },
            )
        },
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

package net.mullvad.mullvadvpn.compose.cell

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.graphics.painter.BitmapPainter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.Alpha40
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemText

@Preview
@Composable
private fun PreviewTunnelingCell() {
    AppTheme {
        Column(
            modifier =
                Modifier.background(color = MaterialTheme.colorScheme.background).padding(20.dp)
        ) {
            SplitTunnelingCell(title = "Mullvad VPN", packageName = "", isSelected = false)
            SplitTunnelingCell(title = "Mullvad VPN", packageName = "", isSelected = true)
        }
    }
}

@Composable
fun SplitTunnelingCell(
    title: String,
    packageName: String?,
    isSelected: Boolean,
    modifier: Modifier = Modifier,
    onResolveIcon: (String) -> Bitmap? = { null },
    onCellClicked: () -> Unit = {}
) {
    var icon by remember(packageName) { mutableStateOf<ImageBitmap?>(null) }
    LaunchedEffect(packageName) {
        launch(Dispatchers.IO) {
            val bitmap = onResolveIcon(packageName ?: "")
            icon = bitmap?.asImageBitmap()
        }
    }
    Row(
        modifier =
            modifier
                .wrapContentHeight()
                .defaultMinSize(minHeight = Dimens.listItemHeightExtra)
                .fillMaxWidth()
                .padding(bottom = Dimens.listItemDivider)
                .background(
                    MaterialTheme.colorScheme.primary
                        .copy(alpha = Alpha40)
                        .compositeOver(MaterialTheme.colorScheme.background)
                )
                .clickable(onClick = onCellClicked)
    ) {
        Image(
            painter = icon?.let { iconImage -> BitmapPainter(iconImage) }
                    ?: painterResource(id = R.drawable.ic_icons_missing),
            contentDescription = null,
            modifier =
                Modifier.padding(start = Dimens.cellStartPadding)
                    .align(Alignment.CenterVertically)
                    .size(width = Dimens.listIconSize, height = Dimens.listIconSize)
        )
        Text(
            text = title,
            style = MaterialTheme.typography.listItemText,
            color = MaterialTheme.colorScheme.onPrimary,
            modifier =
                Modifier.weight(1f)
                    .padding(horizontal = Dimens.mediumPadding, vertical = Dimens.smallPadding)
                    .align(Alignment.CenterVertically)
        )
        Icon(
            painter =
                painterResource(
                    id =
                        if (isSelected) {
                            R.drawable.ic_icons_remove
                        } else {
                            R.drawable.ic_icons_add
                        }
                ),
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onBackground.copy(alpha = Alpha40),
            modifier =
                Modifier.padding(end = Dimens.cellStartPadding)
                    .align(Alignment.CenterVertically)
                    .padding(horizontal = Dimens.loadingSpinnerPadding)
        )
    }
}

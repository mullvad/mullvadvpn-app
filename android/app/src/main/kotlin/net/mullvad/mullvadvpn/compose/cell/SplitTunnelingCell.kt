package net.mullvad.mullvadvpn.compose.cell

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.Alpha40
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemText

@Preview
@Composable
private fun PreviewTunnelingCell() {
    AppTheme {
        SpacedColumn(
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
    backgroundColor: Color =
        MaterialTheme.colorScheme.primary
            .copy(alpha = Alpha40)
            .compositeOver(MaterialTheme.colorScheme.background),
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
    BaseCell(
        iconView = {
            Image(
                painter =
                    icon?.let { iconImage -> BitmapPainter(iconImage) }
                        ?: painterResource(id = R.drawable.ic_icons_missing),
                contentDescription = null,
                modifier =
                    Modifier.align(Alignment.CenterVertically).size(size = Dimens.listIconSize)
            )
        },
        title = {
            Text(
                text = title,
                style = MaterialTheme.typography.listItemText,
                color = MaterialTheme.colorScheme.onPrimary,
                modifier =
                    Modifier.weight(1f)
                        .padding(horizontal = Dimens.mediumPadding)
                        .align(Alignment.CenterVertically)
            )
        },
        bodyView = {
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
                tint =
                    MaterialTheme.colorScheme.onBackground
                        .copy(alpha = Alpha40)
                        .compositeOver(backgroundColor),
                modifier = Modifier.size(size = Dimens.addIconSize)
            )
        },
        onCellClicked = onCellClicked,
        background = backgroundColor,
        modifier = modifier
    )
}

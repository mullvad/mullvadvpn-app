package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ListItem
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue40
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite40
import net.mullvad.mullvadvpn.ui.widget.ApplicationImageView

@Preview
@Composable
fun PreviewTunnelingCell() {
    Column(modifier = Modifier.background(color = MullvadWhite40)) {
        SplitTunnelingCell("Mullvad VPN", "", false) {}
        SplitTunnelingCell("Mullvad VPN", "", true) {}
    }
}

@Composable
fun SplitTunnelingCell(
    title: String,
    packageName: String?,
    isSelected: Boolean,
    onCellClicked: () -> Unit
) {
    val startPadding = dimensionResource(id = R.dimen.cell_left_padding)
    val endPadding = dimensionResource(id = R.dimen.cell_right_padding)
    val iconSize = dimensionResource(id = R.dimen.icon_size)
    Box(
        modifier =
            Modifier.background(MullvadBlue40)
                .padding(top = 1.dp, bottom = 1.dp, start = startPadding, end = endPadding)
                .clickable { onCellClicked() }
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
        ) {
            packageName?.let {
                AndroidView(
                    factory = { context -> ApplicationImageView(context) },
                    update = { applicationImageView ->
                        applicationImageView.packageName = packageName
                    },
                    modifier = Modifier.size(width = iconSize, height = iconSize)
                )
            }
            ListItem(
                text = title,
                isLoading = false,
                iconResourceId =
                    if (isSelected) {
                        R.drawable.ic_icons_remove
                    } else {
                        R.drawable.ic_icons_add
                    },
                background = MullvadBlue40,
                onClick = null
            )
        }
    }
}

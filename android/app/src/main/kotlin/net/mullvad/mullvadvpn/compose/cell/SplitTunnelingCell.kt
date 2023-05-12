package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.viewinterop.AndroidView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ListItem
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.ui.widget.ApplicationImageView

@Preview
@Composable
fun PreviewTunnelingCell() {
    Column(modifier = Modifier.background(color = MaterialTheme.colorScheme.background)) {
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
    Box(
        modifier =
            Modifier.background(MaterialTheme.colorScheme.primaryContainer)
                .padding(vertical = Dimens.listItemDivider)
                .clickable { onCellClicked() }
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier =
                Modifier.padding(start = Dimens.cellLeftPadding, end = Dimens.cellRightPadding)
        ) {
            packageName?.let {
                AndroidView(
                    factory = { context -> ApplicationImageView(context) },
                    update = { applicationImageView ->
                        applicationImageView.packageName = packageName
                    },
                    modifier = Modifier.size(width = Dimens.iconSize, height = Dimens.iconSize)
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
                background = Color.Transparent,
                onClick = null
            )
        }
    }
}

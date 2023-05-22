package net.mullvad.mullvadvpn.compose.cell

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
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.viewinterop.AndroidView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.compose.theme.typeface.listItemText
import net.mullvad.mullvadvpn.ui.widget.ApplicationImageView

@Preview
@Composable
fun PreviewTunnelingCell() {
    AppTheme {
        Column(modifier = Modifier.background(color = MaterialTheme.colorScheme.background)) {
            SplitTunnelingCell("Mullvad VPN", "", false)
            SplitTunnelingCell("Mullvad VPN", "", true)
        }
    }
}

@Composable
fun SplitTunnelingCell(
    title: String,
    packageName: String?,
    isSelected: Boolean,
    modifier: Modifier = Modifier,
    onCellClicked: () -> Unit = {}
) {
    Row(
        modifier =
            modifier
                .wrapContentHeight()
                .defaultMinSize(minHeight = Dimens.listItemHeightExtra)
                .fillMaxWidth()
                .padding(vertical = Dimens.listItemDivider)
                .background(MaterialTheme.colorScheme.primaryContainer)
                .clickable(onClick = onCellClicked)
    ) {
        AndroidView(
            factory = { context -> ApplicationImageView(context) },
            update = { applicationImageView ->
                applicationImageView.packageName = packageName ?: ""
            },
            modifier =
                Modifier.padding(start = Dimens.cellLeftPadding)
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
        Image(
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
            modifier =
                Modifier.padding(end = Dimens.cellRightPadding)
                    .align(Alignment.CenterVertically)
                    .padding(horizontal = Dimens.loadingSpinnerPadding)
        )
    }
}

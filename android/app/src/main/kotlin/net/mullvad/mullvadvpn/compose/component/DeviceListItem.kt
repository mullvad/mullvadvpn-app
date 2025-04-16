package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.TwoRowCell
import net.mullvad.mullvadvpn.lib.common.util.formatDate
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemSubText
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemText

@Composable
fun DeviceListItem(
    device: Device,
    isLoading: Boolean,
    isCurrentDevice: Boolean = false,
    onDeviceRemovalClicked: () -> Unit,
) {
    TwoRowCell(
        titleStyle = MaterialTheme.typography.listItemText,
        titleColor = MaterialTheme.colorScheme.onPrimary,
        subtitleStyle = MaterialTheme.typography.listItemSubText,
        subtitleColor = MaterialTheme.colorScheme.onSurfaceVariant,
        titleText = device.displayName(),
        subtitleText = stringResource(id = R.string.created_x, device.creationDate.formatDate()),
        bodyView = {
            if (isLoading) {
                MullvadCircularProgressIndicatorMedium(
                    modifier = Modifier.padding(Dimens.smallPadding)
                )
            } else if (isCurrentDevice) {
                Text(
                    modifier = Modifier.padding(Dimens.smallPadding),
                    text = stringResource(R.string.current_device),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.labelLarge,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            } else {
                IconButton(onClick = onDeviceRemovalClicked) {
                    Icon(
                        imageVector = Icons.Default.Clear,
                        contentDescription = stringResource(id = R.string.remove_button),
                        tint = MaterialTheme.colorScheme.onPrimary,
                        modifier = Modifier.size(size = Dimens.deleteIconSize),
                    )
                }
            }
        },
        onCellClicked = null,
        endPadding = Dimens.smallPadding,
        minHeight = Dimens.cellHeight,
    )
}

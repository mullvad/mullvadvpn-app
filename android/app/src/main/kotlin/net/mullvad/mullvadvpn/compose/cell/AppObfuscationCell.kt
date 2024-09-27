package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewAppObfuscationCell() {
    AppTheme {
        AppObfuscationCell(
            name = "Obfuscation",
            icon = R.drawable.ic_launcher_game_preview,
            isSelected = true,
            onClick = {},
        )
    }
}

@Composable
fun AppObfuscationCell(
    name: String,
    icon: Int,
    isSelected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    titleStyle: TextStyle = MaterialTheme.typography.labelLarge,
) {
    BaseCell(
        modifier = modifier,
        iconView = {
            SelectableIcon(isEnabled = true, isSelected = isSelected, iconContentDescription = null)
        },
        onCellClicked = onClick,
        headlineContent = {
            Icon(
                painter = painterResource(id = icon),
                contentDescription = null,
                modifier = Modifier.size(24.dp),
                tint = Color.Unspecified,
            )
            Spacer(modifier = Modifier.width(Dimens.mediumPadding))
            BaseCellTitle(title = name, style = titleStyle)
        },
    )
}

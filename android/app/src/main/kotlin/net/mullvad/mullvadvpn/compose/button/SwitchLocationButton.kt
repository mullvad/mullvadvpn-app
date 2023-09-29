package net.mullvad.mullvadvpn.compose.button

import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.Alpha20

@Preview
@Composable
private fun PreviewSwitchLocationButton() {
    AppTheme {
        SpacedColumn {
            SwitchLocationButton(onClick = {}, text = "Switch Location", showChevron = false)
            SwitchLocationButton(onClick = {}, text = "Switch Location", showChevron = true)
        }
    }
}

@Composable
fun SwitchLocationButton(
    modifier: Modifier = Modifier,
    text: String,
    showChevron: Boolean,
    onClick: () -> Unit,
) {
    PrimaryButton(
        onClick = onClick,
        colors =
            ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.inverseSurface.copy(alpha = Alpha20),
                contentColor = MaterialTheme.colorScheme.inverseSurface
            ),
        modifier = modifier,
        text = text,
        icon =
            if (showChevron) {
                {
                    Icon(
                        painter = painterResource(id = R.drawable.icon_chevron),
                        contentDescription = null
                    )
                }
            } else null
    )
}

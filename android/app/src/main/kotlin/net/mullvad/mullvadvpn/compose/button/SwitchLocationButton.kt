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
                containerColor = MaterialTheme.colorScheme.primaryContainer,
                contentColor = MaterialTheme.colorScheme.onPrimaryContainer
            ),
        modifier = modifier,
        text = text,
        trailingIcon =
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

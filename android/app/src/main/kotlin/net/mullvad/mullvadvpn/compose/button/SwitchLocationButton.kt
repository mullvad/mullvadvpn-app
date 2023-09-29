package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
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
    ActionButton(
        onClick = onClick,
        colors =
            ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.inverseSurface.copy(alpha = Alpha20),
                contentColor = MaterialTheme.colorScheme.inverseSurface
            ),
        modifier = modifier
    ) {
        Box(modifier = Modifier.fillMaxWidth().fillMaxHeight().padding(all = Dimens.smallPadding)) {
            Text(
                text = text,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.align(Alignment.Center)
            )
            if (showChevron) {
                Icon(
                    painter = painterResource(id = R.drawable.icon_chevron),
                    contentDescription = null,
                    modifier = Modifier.align(Alignment.CenterEnd)
                )
            }
        }
    }
}

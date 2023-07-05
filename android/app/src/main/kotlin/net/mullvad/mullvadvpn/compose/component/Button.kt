package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonColors
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.Alpha20
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens

@Composable
fun ActionButton(
    onClick: () -> Unit,
    colors: ButtonColors,
    modifier: Modifier = Modifier,
    text: String = "",
    isEnabled: Boolean = true,
    content: @Composable RowScope.() -> Unit = {
        Text(
            text = text,
            textAlign = TextAlign.Center,
            fontSize = 18.sp,
            fontWeight = FontWeight.Bold
        )
    }
) {
    Button(
        onClick = onClick,
        enabled = isEnabled,
        // Required along with defaultMinSize to control size and padding.
        contentPadding = PaddingValues(0.dp),
        modifier =
            modifier
                .height(dimensionResource(id = R.dimen.button_height))
                .defaultMinSize(
                    minWidth = 0.dp,
                    minHeight = dimensionResource(id = R.dimen.button_height)
                )
                .fillMaxWidth(),
        colors = colors,
        shape = MaterialTheme.shapes.small
    ) {
        content()
    }
}

@Preview
@Composable
fun PreviewSwitchLocationButton() {
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

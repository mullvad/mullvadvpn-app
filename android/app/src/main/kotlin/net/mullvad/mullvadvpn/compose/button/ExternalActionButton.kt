package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ButtonColors
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.AlphaVisible
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewExternalActionButton() {
    AppTheme {
        ExternalActionButton(onClick = {}, colors = ButtonDefaults.buttonColors(), text = "Button")
    }
}

@Composable
fun ExternalActionButton(
    onClick: () -> Unit,
    colors: ButtonColors,
    text: String,
    modifier: Modifier = Modifier,
    isEnabled: Boolean = true,
) {
    ActionButton(
        onClick = onClick,
        colors = colors,
        modifier = modifier,
        isEnabled = isEnabled,
    ) {
        Box(modifier = Modifier.fillMaxSize()) {
            Text(
                text = text,
                textAlign = TextAlign.Center,
                style = MaterialTheme.typography.bodyMedium,
                modifier = Modifier.align(Alignment.Center)
            )
            Image(
                painter = painterResource(id = R.drawable.icon_extlink),
                contentDescription = null,
                modifier =
                    Modifier.align(Alignment.CenterEnd)
                        .padding(horizontal = Dimens.smallPadding)
                        .alpha(if (isEnabled) AlphaVisible else AlphaDisabled)
            )
        }
    }
}

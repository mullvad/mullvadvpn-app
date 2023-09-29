package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ButtonColors
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.constraintlayout.compose.ConstraintLayout
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible

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
        ConstraintLayout(modifier = Modifier.fillMaxSize()) {
            val (title, logo) = createRefs()
            Text(
                text = text,
                textAlign = TextAlign.Center,
                style = MaterialTheme.typography.bodyMedium,
                modifier =
                    Modifier.constrainAs(title) {
                        end.linkTo(logo.start)
                        centerTo(parent)
                    }
            )
            Image(
                painter = painterResource(id = R.drawable.icon_extlink),
                contentDescription = null,
                modifier =
                    Modifier.constrainAs(logo) {
                            centerVerticallyTo(parent)
                            end.linkTo(parent.end)
                        }
                        .padding(horizontal = Dimens.smallPadding)
                        .alpha(if (isEnabled) AlphaVisible else AlphaDisabled)
            )
        }
    }
}

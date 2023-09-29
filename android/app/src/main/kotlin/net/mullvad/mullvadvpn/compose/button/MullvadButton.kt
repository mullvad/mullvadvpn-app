package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonColors
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.theme.color.onVariant
import net.mullvad.mullvadvpn.lib.theme.color.variant

@Preview
@Composable
private fun PreviewNegativeButtonEnabled() {
    AppTheme { NegativeButton(onClick = {}, text = "Negative Button") }
}

@Preview
@Composable
private fun PreviewNegativeButtonDisabled() {
    AppTheme { NegativeButton(onClick = {}, text = "Negative Button", isEnabled = false) }
}

@Preview
@Composable
private fun PreviewVariantButtonEnabled() {
    AppTheme { VariantButton(onClick = {}, text = "Variant Button") }
}

@Preview
@Composable
private fun PreviewVariantButtonDisabled() {
    AppTheme { VariantButton(onClick = {}, text = "Variant Button", isEnabled = false) }
}

@Preview
@Composable
private fun PreviewPrimaryButtonEnabled() {
    AppTheme { PrimaryButton(onClick = {}, text = "Primary Button") }
}

@Preview
@Composable
private fun PreviewPrimaryButtonDisabled() {
    AppTheme { PrimaryButton(onClick = {}, text = "Primary Button", isEnabled = false) }
}

@Composable
fun NegativeButton(
    onClick: () -> Unit,
    text: String,
    modifier: Modifier = Modifier,
    background: Color = MaterialTheme.colorScheme.background,
    colors: ButtonColors =
        ButtonDefaults.buttonColors(
            containerColor = MaterialTheme.colorScheme.error,
            contentColor = MaterialTheme.colorScheme.onError,
            disabledContentColor =
                MaterialTheme.colorScheme.onError
                    .copy(alpha = AlphaInactive)
                    .compositeOver(background),
            disabledContainerColor =
                MaterialTheme.colorScheme.error
                    .copy(alpha = AlphaInactive)
                    .compositeOver(background),
        ),
    isEnabled: Boolean = true,
    icon: Int? = null,
    iconContentDescription: String? = null,
) {
    BaseButton(
        onClick = onClick,
        colors = colors,
        text = text,
        modifier = modifier,
        isEnabled = isEnabled,
        icon = icon,
        iconContentDescription = iconContentDescription,
    )
}

@Composable
fun VariantButton(
    onClick: () -> Unit,
    text: String,
    modifier: Modifier = Modifier,
    background: Color = MaterialTheme.colorScheme.background,
    colors: ButtonColors =
        ButtonDefaults.buttonColors(
            containerColor = MaterialTheme.colorScheme.variant,
            contentColor = MaterialTheme.colorScheme.onVariant,
            disabledContentColor =
                MaterialTheme.colorScheme.onVariant
                    .copy(alpha = AlphaInactive)
                    .compositeOver(background),
            disabledContainerColor =
                MaterialTheme.colorScheme.variant
                    .copy(alpha = AlphaInactive)
                    .compositeOver(background),
        ),
    isEnabled: Boolean = true,
    icon: Int? = null,
    iconContentDescription: String? = null,
) {
    BaseButton(
        onClick = onClick,
        colors = colors,
        text = text,
        modifier = modifier,
        isEnabled = isEnabled,
        icon = icon,
        iconContentDescription = iconContentDescription,
    )
}

@Composable
fun PrimaryButton(
    onClick: () -> Unit,
    text: String,
    modifier: Modifier = Modifier,
    colors: ButtonColors = ButtonDefaults.buttonColors(),
    isEnabled: Boolean = true,
    icon: Int? = null,
    iconContentDescription: String? = null,
) {
    BaseButton(
        onClick = onClick,
        colors = colors,
        text = text,
        modifier = modifier,
        isEnabled = isEnabled,
        icon = icon,
        iconContentDescription = iconContentDescription,
    )
}

@Composable
private fun BaseButton(
    onClick: () -> Unit,
    colors: ButtonColors,
    text: String,
    modifier: Modifier = Modifier,
    isEnabled: Boolean = true,
    icon: Int? = null,
    iconContentDescription: String? = null,
) {
    Button(
        onClick = onClick,
        colors = colors,
        enabled = isEnabled,
        // Required along with defaultMinSize to control size and padding.
        contentPadding = PaddingValues(0.dp),
        modifier =
            modifier
                .height(Dimens.buttonHeight)
                .defaultMinSize(minWidth = 0.dp, minHeight = Dimens.buttonHeight)
                .fillMaxWidth(),
        shape = MaterialTheme.shapes.small
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
            icon?.let {
                Image(
                    painter = painterResource(id = icon),
                    contentDescription = iconContentDescription,
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
}

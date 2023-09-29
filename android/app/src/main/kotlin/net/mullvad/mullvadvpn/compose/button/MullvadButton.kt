package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
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
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.Alpha20
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInvisible
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
    colors: ButtonColors =
        ButtonDefaults.buttonColors(
            containerColor = MaterialTheme.colorScheme.error,
            contentColor = MaterialTheme.colorScheme.onError,
            disabledContentColor =
                MaterialTheme.colorScheme.onError
                    .copy(alpha = AlphaInactive)
                    .compositeOver(MaterialTheme.colorScheme.background),
            disabledContainerColor =
                MaterialTheme.colorScheme.error
                    .copy(alpha = AlphaInactive)
                    .compositeOver(MaterialTheme.colorScheme.background),
        ),
    isEnabled: Boolean = true,
    icon: @Composable (() -> Unit)? = null
) {
    BaseButton(
        onClick = onClick,
        colors = colors,
        text = text,
        modifier = modifier,
        isEnabled = isEnabled,
        icon = icon
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
    icon: @Composable (() -> Unit)? = null
) {
    BaseButton(
        onClick = onClick,
        colors = colors,
        text = text,
        modifier = modifier,
        isEnabled = isEnabled,
        icon = icon
    )
}

@Composable
fun PrimaryButton(
    onClick: () -> Unit,
    text: String,
    modifier: Modifier = Modifier,
    colors: ButtonColors =
        ButtonDefaults.buttonColors(
            containerColor = MaterialTheme.colorScheme.primary,
            contentColor = MaterialTheme.colorScheme.onPrimary,
            disabledContentColor =
                MaterialTheme.colorScheme.onPrimary
                    .copy(alpha = Alpha20)
                    .compositeOver(MaterialTheme.colorScheme.background),
            disabledContainerColor =
                MaterialTheme.colorScheme.primary
                    .copy(alpha = AlphaInactive)
                    .compositeOver(MaterialTheme.colorScheme.background),
        ),
    isEnabled: Boolean = true,
    icon: @Composable (() -> Unit)? = null
) {
    BaseButton(
        onClick = onClick,
        colors = colors,
        text = text,
        modifier = modifier,
        isEnabled = isEnabled,
        icon = icon,
    )
}

@Composable
private fun BaseButton(
    onClick: () -> Unit,
    colors: ButtonColors,
    text: String,
    modifier: Modifier = Modifier,
    isEnabled: Boolean = true,
    icon: @Composable (() -> Unit)? = null
) {
    Button(
        onClick = onClick,
        colors = colors,
        enabled = isEnabled,
        contentPadding =
            icon?.let { PaddingValues(horizontal = 0.dp, vertical = Dimens.buttonVerticalPadding) }
                ?: ButtonDefaults.ContentPadding,
        modifier = modifier.wrapContentHeight().fillMaxWidth(),
        shape = MaterialTheme.shapes.small
    ) {
        // Used to center the text
        icon?.let {
            Box(
                modifier = Modifier.padding(horizontal = Dimens.smallPadding).alpha(AlphaInvisible)
            ) {
                icon()
            }
        }
        Text(
            text = text,
            textAlign = TextAlign.Center,
            style = MaterialTheme.typography.bodyMedium,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(1f)
        )
        icon?.let {
            Box(
                modifier =
                    Modifier.padding(horizontal = Dimens.smallPadding)
                        .alpha(if (isEnabled) AlphaVisible else AlphaDisabled)
            ) {
                icon()
            }
        }
    }
}

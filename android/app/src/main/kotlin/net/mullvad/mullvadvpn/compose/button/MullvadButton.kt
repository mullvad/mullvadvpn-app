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
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorSmall
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.Alpha20
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInvisible
import net.mullvad.mullvadvpn.lib.theme.color.errorDisabled
import net.mullvad.mullvadvpn.lib.theme.color.primaryDisabled
import net.mullvad.mullvadvpn.lib.theme.color.tertiaryDisabled

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
            disabledContentColor = MaterialTheme.colorScheme.onError.copy(alpha = Alpha20),
            disabledContainerColor = MaterialTheme.colorScheme.errorDisabled,
        ),
    isEnabled: Boolean = true,
    showSpinner: Boolean = false,
    icon: @Composable (() -> Unit)? = null,
) {
    BaseButton(
        onClick = onClick,
        colors = colors,
        text = text,
        modifier = modifier,
        isEnabled = isEnabled,
        showSpinner = showSpinner,
        trailingIcon = icon,
    )
}

@Composable
fun VariantButton(
    onClick: () -> Unit,
    text: String,
    modifier: Modifier = Modifier,
    background: Color = MaterialTheme.colorScheme.tertiary,
    colors: ButtonColors =
        ButtonDefaults.buttonColors(
            containerColor = background,
            contentColor = MaterialTheme.colorScheme.onTertiary,
            disabledContentColor = MaterialTheme.colorScheme.onTertiary.copy(alpha = Alpha20),
            disabledContainerColor = MaterialTheme.colorScheme.tertiaryDisabled,
        ),
    isEnabled: Boolean = true,
    showSpinner: Boolean = false,
    icon: @Composable (() -> Unit)? = null,
) {
    BaseButton(
        onClick = onClick,
        colors = colors,
        text = text,
        modifier = modifier,
        isEnabled = isEnabled,
        showSpinner = showSpinner,
        trailingIcon = icon,
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
            disabledContentColor = MaterialTheme.colorScheme.onPrimary.copy(alpha = Alpha20),
            disabledContainerColor = MaterialTheme.colorScheme.primaryDisabled,
        ),
    isEnabled: Boolean = true,
    showSpinner: Boolean = false,
    leadingIcon: @Composable (() -> Unit)? = null,
    trailingIcon: @Composable (() -> Unit)? = null,
) {
    BaseButton(
        onClick = onClick,
        colors = colors,
        text = text,
        modifier = modifier,
        isEnabled = isEnabled,
        showSpinner = showSpinner,
        leadingIcon = leadingIcon,
        trailingIcon = trailingIcon,
    )
}

@Composable
private fun BaseButton(
    onClick: () -> Unit,
    colors: ButtonColors,
    text: String,
    modifier: Modifier = Modifier,
    isEnabled: Boolean = true,
    showSpinner: Boolean = false,
    leadingIcon: @Composable (() -> Unit)? = null,
    trailingIcon: @Composable (() -> Unit)? = null,
) {
    val hasIcon = leadingIcon != null || trailingIcon != null
    Button(
        onClick = onClick,
        colors = colors,
        enabled = isEnabled,
        contentPadding =
            if (hasIcon) {
                PaddingValues(vertical = Dimens.buttonVerticalPadding)
            } else {
                ButtonDefaults.ContentPadding
            },
        modifier = modifier.wrapContentHeight().fillMaxWidth(),
        shape = MaterialTheme.shapes.small,
    ) {
        // Used to center the text
        when {
            leadingIcon != null ->
                Box(modifier = Modifier.padding(horizontal = Dimens.smallPadding)) { leadingIcon() }
            trailingIcon != null ->
                // Used to center the text
                Box(
                    modifier =
                        Modifier.padding(horizontal = Dimens.smallPadding).alpha(AlphaInvisible)
                ) {
                    trailingIcon()
                }
        }
        if (showSpinner) {
            MullvadCircularProgressIndicatorSmall()
        } else {
            Text(
                text = text,
                textAlign = TextAlign.Center,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f),
            )
        }
        when {
            trailingIcon != null ->
                Box(modifier = Modifier.padding(horizontal = Dimens.smallPadding)) {
                    trailingIcon()
                }
            leadingIcon != null ->
                // Used to center the text
                Box(
                    modifier =
                        Modifier.padding(horizontal = Dimens.smallPadding).alpha(AlphaInvisible)
                ) {
                    leadingIcon()
                }
        }
    }
}

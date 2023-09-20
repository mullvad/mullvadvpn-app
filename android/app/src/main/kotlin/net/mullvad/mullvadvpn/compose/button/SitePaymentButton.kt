package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.background
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.lib.theme.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewSitePaymentButton() {
    AppTheme {
        SpacedColumn(
            spacing = Dimens.cellVerticalSpacing,
            modifier = Modifier.background(color = MaterialTheme.colorScheme.background)
        ) {
            SitePaymentButton(onClick = {}, isEnabled = true)
            SitePaymentButton(onClick = {}, isEnabled = false)
        }
    }
}

@Composable
fun SitePaymentButton(
    onClick: () -> Unit,
    isEnabled: Boolean,
    modifier: Modifier = Modifier,
    background: Color = MaterialTheme.colorScheme.background,
) {
    ExternalActionButton(
        onClick = onClick,
        modifier = modifier,
        colors =
            ButtonDefaults.buttonColors(
                contentColor = MaterialTheme.colorScheme.onPrimary,
                containerColor = MaterialTheme.colorScheme.surface,
                disabledContentColor =
                    MaterialTheme.colorScheme.onPrimary
                        .copy(alpha = AlphaInactive)
                        .compositeOver(background),
                disabledContainerColor =
                    MaterialTheme.colorScheme.surface
                        .copy(alpha = AlphaDisabled)
                        .compositeOver(background)
            ),
        isEnabled = isEnabled,
        text = stringResource(id = R.string.buy_credit)
    )
}

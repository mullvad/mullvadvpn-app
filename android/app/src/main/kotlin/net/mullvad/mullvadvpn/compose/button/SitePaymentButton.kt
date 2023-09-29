package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.background
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
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
) {
    ExternalButton(
        onClick = onClick,
        modifier = modifier,
        isEnabled = isEnabled,
        text = stringResource(id = R.string.buy_credit)
    )
}

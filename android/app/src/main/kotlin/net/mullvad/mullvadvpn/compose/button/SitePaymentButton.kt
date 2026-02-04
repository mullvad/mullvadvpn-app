package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Preview
@Composable
private fun PreviewSitePaymentButton() {
    AppTheme {
        Column(verticalArrangement = Arrangement.spacedBy(Dimens.mediumSpacer)) {
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
    isLoading: Boolean = false,
) {
    ExternalButton(
        onClick = onClick,
        modifier = modifier,
        isEnabled = isEnabled,
        isLoading = isLoading,
        text = stringResource(id = R.string.buy_credit),
    )
}

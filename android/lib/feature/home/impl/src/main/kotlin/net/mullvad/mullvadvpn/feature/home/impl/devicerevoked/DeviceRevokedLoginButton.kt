package net.mullvad.mullvadvpn.feature.home.impl.devicerevoked

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.lib.ui.designsystem.NegativeButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.VariantButton
import net.mullvad.mullvadvpn.lib.ui.resource.R

@Composable
fun DeviceRevokedLoginButton(onClick: () -> Unit, state: DeviceRevokedUiState) {
    if (state == DeviceRevokedUiState.SECURED) {
        NegativeButton(text = stringResource(id = R.string.go_to_login), onClick = onClick)
    } else {
        VariantButton(text = stringResource(id = R.string.go_to_login), onClick = onClick)
    }
}

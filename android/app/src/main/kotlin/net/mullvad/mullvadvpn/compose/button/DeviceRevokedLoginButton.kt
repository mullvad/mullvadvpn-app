package net.mullvad.mullvadvpn.compose.button

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.state.DeviceRevokedUiState

@Composable
fun DeviceRevokedLoginButton(onClick: () -> Unit, state: DeviceRevokedUiState) {
    if (state == DeviceRevokedUiState.SECURED) {
        NegativeButton(text = stringResource(id = R.string.go_to_login), onClick = onClick)
    } else {
        VariantButton(text = stringResource(id = R.string.go_to_login), onClick = onClick)
    }
}

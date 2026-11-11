package net.mullvad.mullvadvpn.feature.customvpn.impl.confirmation

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customvpn.api.RemoveCustomTunnelConfigConfirmationNavResult
import net.mullvad.mullvadvpn.feature.customvpn.impl.R
import net.mullvad.mullvadvpn.lib.ui.component.dialog.NegativeConfirmationDialog

@Composable
fun RemoveCustomTunnelConfigConfirmDialog(navigator: Navigator, title: String) {
    NegativeConfirmationDialog(
        message = stringResource(id = R.string.custom_tunnels_delete_confirmation, title),
        onConfirm = { navigator.goBack(result = RemoveCustomTunnelConfigConfirmationNavResult) },
        onBack = { navigator.goBack() },
    )
}

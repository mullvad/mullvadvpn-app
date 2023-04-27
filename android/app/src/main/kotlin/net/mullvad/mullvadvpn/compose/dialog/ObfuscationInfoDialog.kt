package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R

@Composable
fun ObfuscationInfoDialog(onDismiss: () -> Unit) {
    InfoDialog(message = stringResource(id = R.string.obfuscation_info), onDismiss = onDismiss)
}

package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R

@Preview
@Composable
private fun PreviewObfuscationInfoDialog() {
    ObfuscationInfoDialog(onDismiss = {})
}

@Composable
fun ObfuscationInfoDialog(onDismiss: () -> Unit) {
    InfoDialog(message = stringResource(id = R.string.obfuscation_info), onDismiss = onDismiss)
}

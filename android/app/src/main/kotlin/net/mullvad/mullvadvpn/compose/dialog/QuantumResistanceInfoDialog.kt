package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R

@Preview
@Composable
private fun PreviewQuantumResistanceInfoDialog() {
    QuantumResistanceInfoDialog(onDismiss = {})
}

@Composable
fun QuantumResistanceInfoDialog(onDismiss: () -> Unit) {
    InfoDialog(
        message = stringResource(id = R.string.quantum_resistant_info_first_paragaph),
        additionalInfo = stringResource(id = R.string.quantum_resistant_info_second_paragaph),
        onDismiss = onDismiss
    )
}

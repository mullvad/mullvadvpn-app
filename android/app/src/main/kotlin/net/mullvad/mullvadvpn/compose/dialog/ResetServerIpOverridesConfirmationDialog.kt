package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.ResetServerIpOverridesConfirmationUiSideEffect
import net.mullvad.mullvadvpn.viewmodel.ResetServerIpOverridesConfirmationViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewResetServerIpOverridesConfirmationDialog() {
    AppTheme { ResetServerIpOverridesConfirmationDialog({}, {}) }
}

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun ResetServerIpOverridesConfirmation(navigator: DestinationsNavigator) {
    val vm: ResetServerIpOverridesConfirmationViewModel = koinViewModel()
    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            ResetServerIpOverridesConfirmationUiSideEffect.OverridesCleared ->
                navigator.navigateUp()
        }
    }
    ResetServerIpOverridesConfirmationDialog(
        onClearAllOverrides = vm::clearAllOverrides,
        navigator::navigateUp
    )
}

@Composable
fun ResetServerIpOverridesConfirmationDialog(
    onClearAllOverrides: () -> Unit,
    onNavigateBack: () -> Unit
) {
    AlertDialog(
        containerColor = MaterialTheme.colorScheme.background,
        confirmButton = {
            NegativeButton(
                modifier = Modifier.fillMaxWidth(),
                text = stringResource(id = R.string.server_ip_overrides_reset_reset_button),
                onClick = onClearAllOverrides
            )
        },
        dismissButton = {
            PrimaryButton(
                modifier = Modifier.fillMaxWidth(),
                text = stringResource(R.string.cancel),
                onClick = onNavigateBack
            )
        },
        title = {
            Text(
                text = stringResource(id = R.string.server_ip_overrides_reset_title),
                color = MaterialTheme.colorScheme.onBackground
            )
        },
        text = {
            Text(
                text = stringResource(id = R.string.server_ip_overrides_reset_body),
                color = MaterialTheme.colorScheme.onBackground,
                style = MaterialTheme.typography.bodySmall,
            )
        },
        onDismissRequest = onNavigateBack
    )
}

package net.mullvad.mullvadvpn.feature.serveripoverride.impl.reset

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.core.nav3.LocalResultStore
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ResetServerIpOverrideConfirmationNavResult
import net.mullvad.mullvadvpn.lib.ui.designsystem.NegativeButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.RESET_SERVER_IP_OVERRIDE_CANCEL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.RESET_SERVER_IP_OVERRIDE_RESET_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewResetServerIpOverridesConfirmationDialog() {
    AppTheme {
        ResetServerIpOverridesConfirmationDialog(onClearAllOverrides = {}, onNavigateBack = {})
    }
}

@Destination<ExternalModuleGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun ResetServerIpOverridesConfirmation(navigator: Navigator) {
    val vm: ResetServerIpOverridesConfirmationViewModel = koinViewModel()
    val resultStore = LocalResultStore.current

    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        val clearSuccessful = when (it) {
            ResetServerIpOverridesConfirmationUiSideEffect.OverridesCleared -> true
            is ResetServerIpOverridesConfirmationUiSideEffect.OverridesError -> false
        }
        navigator.goBack(
            resultStore = resultStore,
            result = ResetServerIpOverrideConfirmationNavResult(clearSuccessful),
        )
    }
    ResetServerIpOverridesConfirmationDialog(
        onClearAllOverrides = vm::clearAllOverrides,
        onNavigateBack = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun ResetServerIpOverridesConfirmationDialog(
    onClearAllOverrides: () -> Unit,
    onNavigateBack: () -> Unit,
) {
    AlertDialog(
        containerColor = MaterialTheme.colorScheme.surface,
        confirmButton = {
            NegativeButton(
                modifier = Modifier.fillMaxWidth().testTag(RESET_SERVER_IP_OVERRIDE_RESET_TEST_TAG),
                text = stringResource(id = R.string.server_ip_overrides_reset_reset_button),
                onClick = onClearAllOverrides,
            )
        },
        dismissButton = {
            PrimaryButton(
                modifier =
                    Modifier.fillMaxWidth().testTag(RESET_SERVER_IP_OVERRIDE_CANCEL_TEST_TAG),
                text = stringResource(R.string.cancel),
                onClick = onNavigateBack,
            )
        },
        title = {
            Text(
                text = stringResource(id = R.string.server_ip_overrides_reset_title),
                color = MaterialTheme.colorScheme.onSurface,
            )
        },
        text = {
            Text(
                text = stringResource(id = R.string.server_ip_overrides_reset_body),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.labelLarge,
            )
        },
        onDismissRequest = onNavigateBack,
    )
}

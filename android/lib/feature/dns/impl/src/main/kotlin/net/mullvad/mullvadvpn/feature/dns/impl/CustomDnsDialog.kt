package net.mullvad.mullvadvpn.feature.dns.impl

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.dns.api.CustomDnsNavKey
import net.mullvad.mullvadvpn.feature.dns.api.CustomDnsNavResult
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InputDialog
import net.mullvad.mullvadvpn.lib.ui.component.textfield.DnsTextField
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewCustomDnsDialogNew() {
    AppTheme {
        CustomDnsDialog(
            state = CustomDnsDialogViewState("1.1.1.1", null, false, false, null),
            onDnsInputChange = {},
            onSaveDnsClick = {},
            onRemoveDnsClick = {},
            onDismiss = {},
        )
    }
}

@Preview
@Composable
private fun PreviewCustomDnsDialogEdit() {
    AppTheme {
        CustomDnsDialog(
            state =
                CustomDnsDialogViewState(
                    input = "1.1.1.1",
                    validationError = null,
                    isAllowLanEnabled = false,
                    isIpv6Enabled = false,
                    index = 0,
                ),
            onDnsInputChange = {},
            onSaveDnsClick = {},
            onRemoveDnsClick = {},
            onDismiss = {},
        )
    }
}

@Preview
@Composable
private fun PreviewCustomDnsDialogEditAllowLanDisabled() {
    AppTheme {
        CustomDnsDialog(
            state = CustomDnsDialogViewState("192.168.1.1", null, false, false, 0),
            onDnsInputChange = {},
            onSaveDnsClick = {},
            onRemoveDnsClick = {},
            onDismiss = {},
        )
    }
}

@Composable
fun CustomDns(navArgs: CustomDnsNavKey, navigator: Navigator) {
    val viewModel = koinViewModel<CustomDnsDialogViewModel> { parametersOf(navArgs) }

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            is CustomDnsDialogSideEffect.Complete ->
                navigator.goBack(result = CustomDnsNavResult.Success(it.isDnsListEmpty))

            CustomDnsDialogSideEffect.Error -> navigator.goBack(result = CustomDnsNavResult.Error)
        }
    }
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    CustomDnsDialog(
        state = state,
        onDnsInputChange = viewModel::onDnsInputChange,
        onSaveDnsClick = viewModel::onSaveDnsClick,
        onRemoveDnsClick = viewModel::onRemoveDnsClick,
        onDismiss = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun CustomDnsDialog(
    state: CustomDnsDialogViewState,
    onDnsInputChange: (String) -> Unit,
    onSaveDnsClick: () -> Unit,
    onRemoveDnsClick: (Int) -> Unit,
    onDismiss: () -> Unit,
) {
    InputDialog(
        title =
            if (state.isNewEntry) {
                stringResource(R.string.add_dns_server_dialog_title)
            } else {
                stringResource(R.string.update_dns_server_dialog_title)
            },
        confirmButtonEnabled = state.isValid(),
        onResetButtonText = stringResource(id = R.string.remove_button),
        messageTextColor = MaterialTheme.colorScheme.error,
        onBack = onDismiss,
        onConfirm = onSaveDnsClick,
        onReset = state.index?.let { { onRemoveDnsClick(state.index) } },
        input = {
            DnsTextField(
                value = state.input,
                isValidValue = state.isValid(),
                onValueChanged = { newDnsValue -> onDnsInputChange(newDnsValue) },
                onSubmit = onSaveDnsClick,
                isEnabled = true,
                placeholderText = stringResource(R.string.custom_dns_hint),
                errorText =
                    when {
                        state.validationError is ValidationError.DuplicateAddress ->
                            stringResource(R.string.duplicate_address_warning)
                        // Ordering is important, as we consider the lan error to have higher
                        // priority than the ipv6 error
                        state.isLocal && !state.isAllowLanEnabled ->
                            stringResource(id = R.string.confirm_local_dns)

                        state.isIpv6 && !state.isIpv6Enabled ->
                            stringResource(id = R.string.confirm_ipv6_dns)

                        else -> null
                    },
                modifier = Modifier.fillMaxWidth(),
            )
        },
    )
}

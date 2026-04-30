package net.mullvad.mullvadvpn.feature.vpnsettings.impl.dns

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
import net.mullvad.mullvadvpn.feature.vpnsettings.api.DnsNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.api.DnsNavResult
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InputDialog
import net.mullvad.mullvadvpn.lib.ui.component.textfield.DnsTextField
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewDnsDialogNew() {
    AppTheme {
        DnsDialog(
            state = DnsDialogViewState("1.1.1.1", null, false, false, null),
            onDnsInputChange = {},
            onSaveDnsClick = {},
            onRemoveDnsClick = {},
            onDismiss = {},
        )
    }
}

@Preview
@Composable
private fun PreviewDnsDialogEdit() {
    AppTheme {
        DnsDialog(
            state =
                DnsDialogViewState(
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
private fun PreviewDnsDialogEditAllowLanDisabled() {
    AppTheme {
        DnsDialog(
            state = DnsDialogViewState("192.168.1.1", null, false, false, 0),
            onDnsInputChange = {},
            onSaveDnsClick = {},
            onRemoveDnsClick = {},
            onDismiss = {},
        )
    }
}

@Composable
fun Dns(navArgs: DnsNavKey, navigator: Navigator) {
    val viewModel = koinViewModel<DnsDialogViewModel> { parametersOf(navArgs) }

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            is DnsDialogSideEffect.Complete ->
                navigator.goBack(result = DnsNavResult.Success(it.isDnsListEmpty))

            DnsDialogSideEffect.Error -> navigator.goBack(result = DnsNavResult.Error)
        }
    }
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    DnsDialog(
        state = state,
        onDnsInputChange = viewModel::onDnsInputChange,
        onSaveDnsClick = viewModel::onSaveDnsClick,
        onRemoveDnsClick = viewModel::onRemoveDnsClick,
        onDismiss = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun DnsDialog(
    state: DnsDialogViewState,
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
                        state.validationError is ValidationError.InvalidAddress.Blank ->
                            stringResource(R.string.invalid_address_blank_warning)
                        state.validationError is ValidationError.InvalidAddress.InvalidIp ->
                            stringResource(R.string.invalid_address_invalid_warning)
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

package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.viewmodel.WireguardCustomPortDialogSideEffect
import net.mullvad.mullvadvpn.viewmodel.WireguardCustomPortDialogViewModel
import org.koin.androidx.compose.koinViewModel

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun WireguardCustomPort(
    @Suppress("UNUSED_PARAMETER") navArg: CustomPortNavArgs,
    backNavigator: ResultBackNavigator<Port?>,
) {
    val viewModel = koinViewModel<WireguardCustomPortDialogViewModel>()

    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            is WireguardCustomPortDialogSideEffect.Success -> backNavigator.navigateBack(it.port)
        }
    }

    CustomPortDialog(
        title =
            stringResource(R.string.custom_port_dialog_title, stringResource(R.string.wireguard)),
        portInput = uiState.portInput,
        isValidInput = uiState.isValidInput,
        showResetToDefault = uiState.showResetToDefault,
        allowedPortRanges = uiState.allowedPortRanges,
        onInputChanged = viewModel::onInputChanged,
        onSavePort = viewModel::onSaveClick,
        onResetPort = viewModel::onResetClick,
        onDismiss = dropUnlessResumed { backNavigator.navigateBack() },
    )
}

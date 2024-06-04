package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.state.DeleteApiAccessMethodUiState
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.DeleteApiAccessMethodConfirmationSideEffect
import net.mullvad.mullvadvpn.viewmodel.DeleteApiAccessMethodConfirmationViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewDeleteApiAccessMethodConfirmationDialog() {
    AppTheme { DeleteApiAccessMethodConfirmationDialog(state = DeleteApiAccessMethodUiState(null)) }
}

@Composable
@Destination(style = DestinationStyle.Dialog::class)
fun DeleteApiAccessMethodConfirmation(
    navigator: ResultBackNavigator<Boolean>,
    apiAccessMethodId: ApiAccessMethodId
) {
    val viewModel =
        koinViewModel<DeleteApiAccessMethodConfirmationViewModel>(
            parameters = { parametersOf(apiAccessMethodId) }
        )
    val state = viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffectCollect(viewModel.uiSideEffect) {
        when (it) {
            is DeleteApiAccessMethodConfirmationSideEffect.Deleted ->
                navigator.navigateBack(result = true)
        }
    }

    DeleteApiAccessMethodConfirmationDialog(
        state = state.value,
        onDelete = viewModel::deleteApiAccessMethod,
        onBack = navigator::navigateBack
    )
}

@Composable
fun DeleteApiAccessMethodConfirmationDialog(
    state: DeleteApiAccessMethodUiState,
    onDelete: () -> Unit = {},
    onBack: () -> Unit = {}
) {
    AlertDialog(
        onDismissRequest = onBack,
        icon = {
            Icon(
                modifier = Modifier.fillMaxWidth().height(Dimens.dialogIconHeight),
                painter = painterResource(id = R.drawable.icon_alert),
                contentDescription = stringResource(id = R.string.remove_button),
                tint = Color.Unspecified
            )
        },
        title = {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Text(
                    text =
                        stringResource(
                            id = R.string.delete_method_question,
                        )
                )
                if (state.deleteError != null) {
                    Text(
                        text = stringResource(id = R.string.error_occurred),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.error,
                        modifier = Modifier.padding(top = Dimens.smallPadding)
                    )
                }
            }
        },
        dismissButton = {
            PrimaryButton(
                modifier = Modifier.focusRequester(FocusRequester()),
                onClick = onBack,
                text = stringResource(id = R.string.cancel)
            )
        },
        confirmButton = {
            NegativeButton(onClick = onDelete, text = stringResource(id = R.string.delete))
        },
        containerColor = MaterialTheme.colorScheme.background
    )
}

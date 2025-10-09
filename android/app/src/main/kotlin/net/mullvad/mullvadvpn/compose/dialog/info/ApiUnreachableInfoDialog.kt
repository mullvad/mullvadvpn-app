package net.mullvad.mullvadvpn.compose.dialog.info

import android.content.ActivityNotFoundException
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import co.touchlab.kermit.Logger
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.state.ApiUnreachableUiState
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.EmailData
import net.mullvad.mullvadvpn.compose.util.SendEmail
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.provider.logUri
import net.mullvad.mullvadvpn.viewmodel.ApiUnreachableSideEffect
import net.mullvad.mullvadvpn.viewmodel.ApiUnreachableViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewApiUnreachableInfoDialog() {
    AppTheme {
        ApiUnreachableInfoDialog(
            uiState = ApiUnreachableUiState(true),
            onEnableAllApiMethods = {},
            onSendEmail = {},
            onDismiss = {},
        )
    }
}

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun ApiUnreachableInfo(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<ApiUnreachableViewModel>()

    val launcher = rememberLauncherForActivityResult(SendEmail()) {}
    val context = LocalContext.current
    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            is ApiUnreachableSideEffect.SendEmail -> {
                val emailData =
                    EmailData(
                        to = listOf(sideEffect.address),
                        subject = sideEffect.subject,
                        attachment = context.logUri(sideEffect.logs),
                    )
                try {
                    launcher.launch(emailData)
                } catch (e: ActivityNotFoundException) {
                    Logger.e("No email client found", e)
                }
            }
        }
    }

    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    ApiUnreachableInfoDialog(
        uiState = uiState,
        onEnableAllApiMethods = viewModel::enableAllApiAccess,
        onSendEmail = viewModel::sendProblemReportEmail,
        onDismiss = navigator::navigateUp,
    )
}

@Composable
fun ApiUnreachableInfoDialog(
    uiState: ApiUnreachableUiState,
    onEnableAllApiMethods: () -> Unit,
    onSendEmail: () -> Unit,
    onDismiss: () -> Unit,
) {
    InfoDialog(
        title = stringResource(id = R.string.unable_to_reach_api_dialog_title),
        message = stringResource(id = R.string.unable_to_reach_api_dialog_message_first),
        additionalInfo = stringResource(id = R.string.unable_to_reach_api_dialog_message_second),
        showIcon = false,
        confirmButton = {
            Column {
                if (uiState.showEnableAllAccessMethodsButton) {
                    PrimaryButton(
                        modifier = Modifier.wrapContentHeight().fillMaxWidth(),
                        text = stringResource(R.string.enable_all_methods),
                        onClick = { onEnableAllApiMethods() },
                    )
                }
                PrimaryButton(
                    modifier = Modifier.wrapContentHeight().fillMaxWidth(),
                    text = stringResource(R.string.send_email),
                    onClick = onSendEmail,
                )
                PrimaryButton(
                    modifier = Modifier.wrapContentHeight().fillMaxWidth(),
                    text = stringResource(R.string.got_it),
                    onClick = onDismiss,
                )
            }
        },
        onDismiss = onDismiss,
    )
}

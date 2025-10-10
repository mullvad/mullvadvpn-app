package net.mullvad.mullvadvpn.compose.dialog.info

import android.content.ActivityNotFoundException
import android.os.Parcelable
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
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.state.ApiUnreachableUiState
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.EmailData
import net.mullvad.mullvadvpn.compose.util.SendEmail
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.provider.createShareLogFile
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

@Parcelize
enum class LoginAction : Parcelable {
    LOGIN,
    CREATE_ACCOUNT,
}

@Parcelize data class ApiUnreachableInfoDialogNavArgs(val action: LoginAction) : Parcelable

sealed interface ApiUnreachableInfoDialogResult : Parcelable {
    @Parcelize
    data class Success(val arg: ApiUnreachableInfoDialogNavArgs) : ApiUnreachableInfoDialogResult

    @Parcelize data object Error : ApiUnreachableInfoDialogResult
}

@Destination<RootGraph>(
    style = DestinationStyle.Dialog::class,
    navArgs = ApiUnreachableInfoDialogNavArgs::class,
)
@Composable
fun ApiUnreachableInfo(navigator: ResultBackNavigator<ApiUnreachableInfoDialogResult>) {
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
                        attachment = context.createShareLogFile(sideEffect.logs),
                    )
                try {
                    launcher.launch(emailData)
                } catch (e: ActivityNotFoundException) {
                    Logger.e("No email client found", e)
                }
            }
            is ApiUnreachableSideEffect.EnableAllApiAccessMethods ->
                navigator.navigateBack(result = sideEffect.toResult())
        }
    }

    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    ApiUnreachableInfoDialog(
        uiState = uiState,
        onEnableAllApiMethods = viewModel::enableAllApiAccess,
        onSendEmail = viewModel::sendProblemReportEmail,
        onDismiss = navigator::navigateBack,
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
                        onClick = onEnableAllApiMethods,
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

private fun ApiUnreachableSideEffect.EnableAllApiAccessMethods.toResult() =
    when (this) {
        ApiUnreachableSideEffect.EnableAllApiAccessMethods.Error ->
            ApiUnreachableInfoDialogResult.Error
        is ApiUnreachableSideEffect.EnableAllApiAccessMethods.Success ->
            ApiUnreachableInfoDialogResult.Success(navArgs)
    }

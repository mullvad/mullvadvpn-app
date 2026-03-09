package net.mullvad.mullvadvpn.feature.login.impl.apiunreachable

import android.content.ActivityNotFoundException
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.core.nav3.LocalResultStore
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.login.api.ApiUnreachableInfoDialogNavArgs
import net.mullvad.mullvadvpn.feature.login.api.ApiUnreachableInfoDialogResult
import net.mullvad.mullvadvpn.feature.login.api.LoginAction
import net.mullvad.mullvadvpn.feature.problemreport.impl.provider.createShareLogFile
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog
import net.mullvad.mullvadvpn.lib.ui.component.textfield.ErrorSupportingText
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewApiUnreachableInfoDialog() {
    AppTheme {
        ApiUnreachableInfoDialog(
            state =
                ApiUnreachableUiState(
                    showEnableAllAccessMethodsButton = true,
                    noEmailAppAvailable = true,
                    loginAction = LoginAction.LOGIN,
                ),
            onEnableAllApiMethods = {},
            onSendEmail = {},
            onDismiss = {},
        )
    }
}

@Composable
fun ApiUnreachableInfo(navigator: Navigator, navArgs: ApiUnreachableInfoDialogNavArgs) {
    val viewModel = koinViewModel<ApiUnreachableViewModel> { parametersOf(navArgs) }

    val launcher = rememberLauncherForActivityResult(SendEmail()) {}
    val context = LocalContext.current
    val resultStore = LocalResultStore.current
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
                    Logger.e("No email app found", e)
                    viewModel.noEmailAppAvailable()
                }
            }
            is ApiUnreachableSideEffect.EnableAllApiAccessMethods ->
                navigator.goBack(resultStore, result = sideEffect.toResult())
        }
    }

    val state by viewModel.uiState.collectAsStateWithLifecycle()

    ApiUnreachableInfoDialog(
        state = state,
        onEnableAllApiMethods = viewModel::enableAllApiAccess,
        onSendEmail = viewModel::sendProblemReportEmail,
        onDismiss = navigator::goBack,
    )
}

@Composable
fun ApiUnreachableInfoDialog(
    state: ApiUnreachableUiState,
    onEnableAllApiMethods: () -> Unit,
    onSendEmail: () -> Unit,
    onDismiss: () -> Unit,
) {
    InfoDialog(
        title = stringResource(id = R.string.unable_to_reach_api),
        message =
            buildAnnotatedString {
                append(
                    stringResource(
                        id =
                            when (state.loginAction) {
                                LoginAction.LOGIN ->
                                    R.string.unable_to_reach_api_dialog_message_first_login

                                LoginAction.CREATE_ACCOUNT ->
                                    R.string.unable_to_reach_api_dialog_message_first_create_account
                            }
                    )
                )
                append(stringResource(id = R.string.unable_to_reach_api_dialog_message_second))
                val firstItem =
                    stringResource(id = R.string.unable_to_reach_api_dialog_message_list_first)
                val secondItem =
                    stringResource(id = R.string.unable_to_reach_api_dialog_message_list_second)
                val thirdItem =
                    stringResource(id = R.string.unable_to_reach_api_dialog_message_list_third)
                withBulletList {
                    withBulletListItem { append(firstItem) }
                    withBulletListItem { append(secondItem) }
                    if (state.showEnableAllAccessMethodsButton) {
                        withBulletListItem { append(thirdItem) }
                    }
                }
            },
        additionalInfo = stringResource(id = R.string.unable_to_reach_api_dialog_message_third),
        showIcon = false,
        confirmButton = {
            Column {
                if (state.showEnableAllAccessMethodsButton) {
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
                if (state.noEmailAppAvailable) {
                    ErrorSupportingText(
                        stringResource(id = R.string.no_email_app_available),
                        modifier = Modifier.padding(bottom = Dimens.smallPadding),
                    )
                }
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

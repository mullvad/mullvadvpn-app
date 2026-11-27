package net.mullvad.mullvadvpn.feature.home.impl.welcome

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.LayoutDirection
import androidx.credentials.CreatePasswordRequest
import androidx.credentials.CredentialManager
import androidx.credentials.exceptions.CreateCredentialException
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.account.api.AccountNavKey
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingNavKey
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingStep
import net.mullvad.mullvadvpn.feature.settings.api.SettingsNavKey
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.common.compose.createCopyToClipboardHandle
import net.mullvad.mullvadvpn.lib.common.compose.createOpenAccountPageHook
import net.mullvad.mullvadvpn.lib.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.common.util.groupWithSpaces
import net.mullvad.mullvadvpn.lib.ui.component.CopyAnimatedIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.lib.ui.designsystem.NegativeButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.GET_STARTED_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import org.koin.androidx.compose.koinViewModel

@Preview("Loading|Content|TunnelConnected")
@Composable
private fun PreviewWelcomeScreen(
    @PreviewParameter(WelcomeScreenUiStatePreviewParameterProvider::class)
    state: Lc<Unit, WelcomeUiState>,
) {
    AppTheme {
        WelcomeScreen(
            state = state,
            onSettingsClick = {},
            onAccountClick = {},
            onGetStartedClick = {},
            onDisconnectClick = {},
        )
    }
}

@Composable
fun Welcome(navigator: Navigator) {
    val vm = koinViewModel<WelcomeViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current
    val resources = LocalResources.current
    val openAccountPage = LocalUriHandler.current.createOpenAccountPageHook()
    CollectSideEffectWithLifecycle(sideEffect = vm.uiSideEffect, Lifecycle.State.RESUMED) {
        uiSideEffect ->
        when (uiSideEffect) {
            is WelcomeViewModel.UiSideEffect.OpenAccountView -> openAccountPage(uiSideEffect.token)
            WelcomeViewModel.UiSideEffect.GenericError ->
                snackbarHostState.showSnackbarImmediately(
                    message = resources.getString(R.string.error_occurred),
                )
            is WelcomeViewModel.UiSideEffect.StoreCredentialsRequest -> {
                // UserId is not allowed to be empty
                val createPasswordRequest =
                    CreatePasswordRequest(id = "-", password = uiSideEffect.accountNumber.value)
                val credentialsManager = CredentialManager.create(context)
                try {
                    credentialsManager.createCredential(context, createPasswordRequest)
                } catch (_: CreateCredentialException) {
                    Logger.w("Unable to create Credentials")
                }
            }
        }
    }

    WelcomeScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onSettingsClick = dropUnlessResumed { navigator.navigate(SettingsNavKey) },
        onAccountClick = dropUnlessResumed { navigator.navigate(AccountNavKey) },
        onDisconnectClick = vm::onDisconnectClick,
        onGetStartedClick =
            dropUnlessResumed { navigator.navigate(OnboardingNavKey(OnboardingStep.AddTime)) },
    )
}

@Composable
fun WelcomeScreen(
    state: Lc<Unit, WelcomeUiState>,
    snackbarHostState: SnackbarHostState = SnackbarHostState(),
    onSettingsClick: () -> Unit,
    onAccountClick: () -> Unit,
    onGetStartedClick: () -> Unit,
    onDisconnectClick: () -> Unit,
) {
    val scrollState = rememberScrollState()

    ScaffoldWithTopBar(
        topBarColor = MaterialTheme.colorScheme.background,
        iconTintColor = MaterialTheme.colorScheme.onBackground,
        onSettingsClicked = onSettingsClick,
        onAccountClicked = onAccountClick,
        snackbarHostState = snackbarHostState,
    ) {
        Column(
            modifier =
                Modifier.fillMaxSize()
                    .background(color = MaterialTheme.colorScheme.surface)
                    .padding(it)
                    .verticalScroll(scrollState)
                    .drawVerticalScrollbar(
                        state = scrollState,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                    ),
        ) {
            // Welcome info area
            WelcomeInfo(snackbarHostState, state)

            Spacer(modifier = Modifier.weight(1f))

            // Button area
            if (state is Lc.Content) {
                ButtonPanel(
                    showDisconnectButton = state.value.tunnelState.isSecured(),
                    onGetStartedClick = onGetStartedClick,
                    onDisconnectClick = onDisconnectClick,
                )
            }
        }
    }
}

@Composable
private fun WelcomeInfo(
    snackbarHostState: SnackbarHostState,
    state: Lc<Unit, WelcomeUiState>,
) {
    Column {
        Text(
            text = stringResource(id = R.string.new_account_created),
            modifier =
                Modifier.fillMaxWidth()
                    .padding(
                        top = Dimens.screenTopMargin,
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                    ),
            style = MaterialTheme.typography.headlineSmall,
            color = MaterialTheme.colorScheme.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        when (state) {
            is Lc.Loading ->
                MullvadCircularProgressIndicatorMedium(
                    modifier =
                        Modifier.padding(
                            horizontal = Dimens.sideMargin,
                            vertical = Dimens.smallPadding,
                        ),
                )
            is Lc.Content -> {
                // Content is English or numbers so we should keep Ltr direction.
                CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Ltr) {
                    // Account number
                    AccountNumberRow(snackbarHostState, state.value)
                }
                Text(
                    modifier = Modifier.padding(horizontal = Dimens.sideMargin),
                    text = stringResource(R.string.account_number_explanation),
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Spacer(Modifier.height(Dimens.largeSpacer))
                Text(
                    modifier = Modifier.padding(horizontal = Dimens.sideMargin),
                    text = stringResource(R.string.new_device_added_to_your_account),
                    style = MaterialTheme.typography.titleMedium,
                )
                Text(
                    modifier = Modifier.padding(horizontal = Dimens.sideMargin),
                    text = stringResource(R.string.up_to_five_devices_explanation),
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Spacer(Modifier.height(Dimens.largeSpacer))
                state.value.deviceName?.let { deviceName -> DeviceName(deviceName) }
            }
        }
    }
}

@Composable
private fun AccountNumberRow(snackbarHostState: SnackbarHostState, state: WelcomeUiState) {
    val copiedAccountNumberMessage = stringResource(id = R.string.copied_mullvad_account_number)
    val copyToClipboard =
        createCopyToClipboardHandle(snackbarHostState = snackbarHostState, isSensitive = true)
    val onCopyToClipboard = {
        copyToClipboard(state.accountNumber?.value.orEmpty(), copiedAccountNumberMessage)
    }

    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
        modifier =
            Modifier.fillMaxWidth()
                .clickable(onClick = onCopyToClipboard)
                .padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    top = Dimens.mediumSpacer,
                ),
    ) {
        Text(
            text = state.accountNumber?.value?.groupWithSpaces().orEmpty(),
            modifier = Modifier.weight(1f),
            style = MaterialTheme.typography.headlineSmall,
            fontFamily = FontFamily.Monospace,
            color = MaterialTheme.colorScheme.onSurface,
        )

        CopyAnimatedIconButton(onCopyToClipboard)
    }
}

@Composable
fun DeviceName(deviceName: String) {
    Text(
        modifier = Modifier.padding(horizontal = Dimens.sideMargin),
        style = MaterialTheme.typography.bodyLarge,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        text =
            buildAnnotatedString {
                val text = stringResource(R.string.this_devices_name, deviceName)
                append(text)

                val nameStart = text.indexOf(deviceName)
                addStyle(
                    style =
                        SpanStyle(
                            color = MaterialTheme.colorScheme.onSurface,
                        ),
                    start = nameStart,
                    end = nameStart + deviceName.length,
                )
            },
    )
}

@Composable
private fun ButtonPanel(
    showDisconnectButton: Boolean,
    onGetStartedClick: () -> Unit,
    onDisconnectClick: () -> Unit,
) {
    Column(
        modifier =
            Modifier.fillMaxWidth()
                .padding(
                    top = Dimens.mediumPadding,
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                ),
    ) {
        Spacer(modifier = Modifier.padding(top = Dimens.screenTopMargin))
        if (showDisconnectButton) {
            NegativeButton(
                onClick = onDisconnectClick,
                text = stringResource(id = R.string.disconnect),
                modifier = Modifier.padding(bottom = Dimens.buttonSpacing),
            )
        }
        PrimaryButton(
            onClick = onGetStartedClick,
            text = stringResource(id = R.string.get_started),
            modifier =
                Modifier.padding(bottom = Dimens.screenBottomMarginNew)
                    .testTag(GET_STARTED_BUTTON_TEST_TAG),
        )
    }
}

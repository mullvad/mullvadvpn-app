package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material.icons.filled.OpenInNew
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.NavGraphs
import com.ramcosta.composedestinations.generated.destinations.LoginDestination
import com.ramcosta.composedestinations.generated.destinations.SplashDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeout
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.constant.DAEMON_READY_TIMEOUT_MS
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.util.appendHideNavOnPlayBuild
import net.mullvad.mullvadvpn.viewmodel.PrivacyDisclaimerUiSideEffect
import net.mullvad.mullvadvpn.viewmodel.PrivacyDisclaimerViewModel
import net.mullvad.mullvadvpn.viewmodel.PrivacyDisclaimerViewState
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewPrivacyDisclaimerScreen() {
    AppTheme {
        PrivacyDisclaimerScreen(
            PrivacyDisclaimerViewState(isStartingService = false, isPlayBuild = false),
            {},
        )
    }
}

@Destination<RootGraph>
@Composable
fun PrivacyDisclaimer(navigator: DestinationsNavigator) {
    val viewModel: PrivacyDisclaimerViewModel = koinViewModel()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    val context = LocalContext.current
    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            PrivacyDisclaimerUiSideEffect.NavigateToLogin ->
                navigator.navigate(LoginDestination(null)) {
                    launchSingleTop = true
                    popUpTo(NavGraphs.root) { inclusive = true }
                }
            PrivacyDisclaimerUiSideEffect.StartService ->
                launch {
                    try {
                        withTimeout(DAEMON_READY_TIMEOUT_MS) {
                            (context as MainActivity).bindService()
                        }
                        viewModel.onServiceStartedSuccessful()
                    } catch (e: CancellationException) {
                        // Timeout
                        viewModel.onServiceStartedTimeout()
                    }
                }
            PrivacyDisclaimerUiSideEffect.NavigateToSplash ->
                navigator.navigate(SplashDestination) {
                    launchSingleTop = true
                    popUpTo(NavGraphs.root) { inclusive = true }
                }
        }
    }
    PrivacyDisclaimerScreen(state, viewModel::setPrivacyDisclosureAccepted)
}

@Composable
fun PrivacyDisclaimerScreen(state: PrivacyDisclaimerViewState, onAcceptClicked: () -> Unit) {
    val topColor = MaterialTheme.colorScheme.primary
    ScaffoldWithTopBar(topBarColor = topColor, onAccountClicked = null, onSettingsClicked = null) {
        val scrollState = rememberScrollState()
        Column(
            Modifier.padding(it)
                .fillMaxSize()
                .background(color = MaterialTheme.colorScheme.surface)
                .verticalScroll(scrollState)
                .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenVerticalMargin)
                .drawVerticalScrollbar(
                    state = scrollState,
                    color = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaScrollbar),
                ),
            verticalArrangement = Arrangement.SpaceBetween,
        ) {
            Content(state.isPlayBuild)

            ButtonPanel(state.isStartingService, onAcceptClicked)
        }
    }
}

@Composable
private fun Content(isPlayBuild: Boolean) {
    Column {
        Text(
            text = stringResource(id = R.string.privacy_disclaimer_title),
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )

        Spacer(modifier = Modifier.height(Dimens.smallPadding))

        Text(
            text = stringResource(id = R.string.privacy_disclaimer_body_first_paragraph),
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.bodySmall,
        )

        Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))

        Text(
            text = stringResource(id = R.string.privacy_disclaimer_body_second_paragraph),
            color = MaterialTheme.colorScheme.onSurface,
            style = MaterialTheme.typography.bodySmall,
        )

        Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))

        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = buildPrivacyPolicyAnnotatedString(isPlayBuild),
                modifier = Modifier.padding(end = Dimens.miniPadding),
                style = MaterialTheme.typography.bodySmall,
            )

            Icon(
                imageVector = Icons.AutoMirrored.Filled.OpenInNew,
                contentDescription = null,
                modifier =
                    Modifier.align(Alignment.CenterVertically).size(Dimens.privacyPolicyIconSize),
                tint = MaterialTheme.colorScheme.onSurface,
            )
        }
    }
}

@Composable
private fun buildPrivacyPolicyAnnotatedString(isPlayBuild: Boolean) = buildAnnotatedString {
    withLink(
        LinkAnnotation.Url(
            stringResource(R.string.privacy_policy_url).appendHideNavOnPlayBuild(isPlayBuild)
        )
    ) {
        withStyle(
            style =
                SpanStyle(
                    color = MaterialTheme.colorScheme.onSurface,
                    textDecoration = TextDecoration.Underline,
                )
        ) {
            append(stringResource(id = R.string.privacy_policy_label))
        }
    }
}

@Composable
private fun ButtonPanel(isStartingService: Boolean, onAcceptClicked: () -> Unit) {
    Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
        if (isStartingService) {
            MullvadCircularProgressIndicatorMedium()
        } else {
            PrimaryButton(
                text = stringResource(id = R.string.agree_and_continue),
                onClick = onAcceptClicked,
            )
        }
    }
}

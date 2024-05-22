package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import android.net.Uri
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.ClickableText
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.popUpTo
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeout
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.NavGraphs
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.destinations.LoginDestination
import net.mullvad.mullvadvpn.compose.destinations.SplashDestination
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.compose.util.toDp
import net.mullvad.mullvadvpn.constant.DAEMON_READY_TIMEOUT_MS
import net.mullvad.mullvadvpn.lib.common.util.openLink
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
            {}
        )
    }
}

@Destination
@Composable
fun PrivacyDisclaimer(
    navigator: DestinationsNavigator,
) {
    val viewModel: PrivacyDisclaimerViewModel = koinViewModel()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    val context = LocalContext.current
    LaunchedEffectCollect(viewModel.uiSideEffect) {
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
                            (context as MainActivity).startServiceSuspend()
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
    PrivacyDisclaimerScreen(
        state,
        { openPrivacyPolicy(context, state.isPlayBuild) },
        viewModel::setPrivacyDisclosureAccepted
    )
}

@Composable
fun PrivacyDisclaimerScreen(
    state: PrivacyDisclaimerViewState,
    onPrivacyPolicyLinkClicked: () -> Unit,
    onAcceptClicked: () -> Unit,
) {
    val topColor = MaterialTheme.colorScheme.primary
    ScaffoldWithTopBar(topBarColor = topColor, onAccountClicked = null, onSettingsClicked = null) {
        val scrollState = rememberScrollState()
        Column(
            Modifier.padding(it)
                .fillMaxSize()
                .background(color = MaterialTheme.colorScheme.background)
                .verticalScroll(scrollState)
                .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenVerticalMargin)
                .drawVerticalScrollbar(
                    state = scrollState,
                    color = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaScrollbar)
                ),
            verticalArrangement = Arrangement.SpaceBetween
        ) {
            Content(onPrivacyPolicyLinkClicked)

            ButtonPanel(state.isStartingService, onAcceptClicked)
        }
    }
}

@Composable
private fun Content(onPrivacyPolicyLinkClicked: () -> Unit) {
    Column {
        Text(
            text = stringResource(id = R.string.privacy_disclaimer_title),
            style = MaterialTheme.typography.headlineSmall,
            color = MaterialTheme.colorScheme.onBackground,
            fontWeight = FontWeight.Bold
        )

        val fontSize = 14.sp
        Text(
            text = stringResource(id = R.string.privacy_disclaimer_body_first_paragraph),
            fontSize = fontSize,
            color = MaterialTheme.colorScheme.onBackground,
            modifier = Modifier.padding(top = 10.dp)
        )

        Spacer(modifier = Modifier.height(fontSize.toDp() + Dimens.smallPadding))

        Text(
            text = stringResource(id = R.string.privacy_disclaimer_body_second_paragraph),
            fontSize = fontSize,
            color = MaterialTheme.colorScheme.onBackground,
        )

        Row(modifier = Modifier.padding(top = 10.dp)) {
            ClickableText(
                text = AnnotatedString(stringResource(id = R.string.privacy_policy_label)),
                onClick = { onPrivacyPolicyLinkClicked() },
                style =
                    TextStyle(
                        fontSize = 12.sp,
                        color = Color.White,
                        textDecoration = TextDecoration.Underline
                    )
            )

            Image(
                painter = painterResource(id = R.drawable.icon_extlink),
                contentDescription = null,
                modifier =
                    Modifier.align(Alignment.CenterVertically)
                        .padding(start = 2.dp, top = 2.dp)
                        .width(10.dp)
                        .height(10.dp)
            )
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
                onClick = onAcceptClicked::invoke
            )
        }
    }
}

private fun openPrivacyPolicy(context: Context, isPlayBuild: Boolean) {
    context.openLink(
        Uri.parse(
            context.resources
                .getString(R.string.privacy_policy_url)
                .appendHideNavOnPlayBuild(isPlayBuild)
        )
    )
}

package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.ClickableText
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
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
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
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
import net.mullvad.mullvadvpn.compose.util.toDp
import net.mullvad.mullvadvpn.constant.DAEMON_READY_TIMEOUT_MS
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.viewmodel.PrivacyDisclaimerUiSideEffect
import net.mullvad.mullvadvpn.viewmodel.PrivacyDisclaimerViewModel
import net.mullvad.mullvadvpn.viewmodel.PrivacyDisclaimerViewState
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewPrivacyDisclaimerScreen() {
    AppTheme { PrivacyDisclaimerScreen(PrivacyDisclaimerViewState(false), {}, {}) }
}

@Destination
@Composable
fun PrivacyDisclaimer(
    navigator: DestinationsNavigator,
) {
    val viewModel: PrivacyDisclaimerViewModel = koinViewModel()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    LaunchedEffect(Unit) {
        viewModel.uiSideEffect.collect {
            when (it) {
                PrivacyDisclaimerUiSideEffect.NavigateToLogin -> {
                    navigator.navigate(LoginDestination(null)) {
                        launchSingleTop = true
                        popUpTo(NavGraphs.root) { inclusive = true }
                    }
                }
                PrivacyDisclaimerUiSideEffect.StartService -> {
                    scope.launch {
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
                }
                PrivacyDisclaimerUiSideEffect.NavigateToSplash -> {
                    navigator.navigate(SplashDestination) {
                        launchSingleTop = true
                        popUpTo(NavGraphs.root) { inclusive = true }
                    }
                }
            }
        }
    }
    PrivacyDisclaimerScreen(state, {}, viewModel::setPrivacyDisclosureAccepted)
}

@Suppress("LongMethod")
@Composable
fun PrivacyDisclaimerScreen(
    state: PrivacyDisclaimerViewState,
    onPrivacyPolicyLinkClicked: () -> Unit,
    onAcceptClicked: () -> Unit,
) {
    val topColor = MaterialTheme.colorScheme.primary
    ScaffoldWithTopBar(topBarColor = topColor, onAccountClicked = null, onSettingsClicked = null) {
        ConstraintLayout(
            modifier =
                Modifier.padding(it)
                    .fillMaxSize()
                    .background(color = MaterialTheme.colorScheme.background)
        ) {
            val (body, actionButtons) = createRefs()
            val sideMargin = Dimens.sideMargin
            val scrollState = rememberScrollState()

            Column(
                modifier =
                    Modifier.constrainAs(body) {
                            top.linkTo(parent.top)
                            start.linkTo(parent.start)
                            end.linkTo(parent.end)
                            bottom.linkTo(actionButtons.top)
                            width = Dimension.fillToConstraints
                            height = Dimension.fillToConstraints
                        }
                        .drawVerticalScrollbar(
                            state = scrollState,
                            color = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaScrollbar)
                        )
                        .verticalScroll(scrollState)
                        .padding(sideMargin),
            ) {
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

            Column(
                modifier =
                    Modifier.constrainAs(actionButtons) {
                        top.linkTo(body.bottom, margin = sideMargin)
                        start.linkTo(parent.start, margin = sideMargin)
                        end.linkTo(parent.end, margin = sideMargin)
                        bottom.linkTo(parent.bottom, margin = sideMargin)
                        width = Dimension.fillToConstraints
                        height = Dimension.preferredWrapContent
                    },
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                if (state.isStartingService) {
                    MullvadCircularProgressIndicatorMedium()
                } else {
                    PrimaryButton(
                        text = stringResource(id = R.string.agree_and_continue),
                        onClick = onAcceptClicked::invoke
                    )
                }
            }
        }
    }
}

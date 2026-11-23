package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.home.api.ConnectNavKey
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingNavKey
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingStep
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_NEXT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewGoodToKnowBlockedContentScreen() {
    AppTheme {
        GoodToKnowScreen(
            state = GoodToKnowUiState(GoodToKnowPage.BLOCKED_CONTENT),
            onBackClick = null,
            onNextClick = {},
            onSkipClick = {},
        )
    }
}

@Preview
@Composable
private fun PreviewGoodToKnowExcludingAppsScreen() {
    AppTheme {
        GoodToKnowScreen(
            state = GoodToKnowUiState(GoodToKnowPage.EXCLUDING_APPS),
            onBackClick = {},
            onNextClick = {},
            onSkipClick = {},
        )
    }
}

@Composable
fun GoodToKnow1(navigator: Navigator) {
    GoodToKnow(
        step = GoodToKnowStep.GoodToKnow1,
        nextStep = GoodToKnowStep.GoodToKnow2,
        navigator = navigator,
    )
}

@Composable
fun GoodToKnow2(navigator: Navigator) {
    GoodToKnow(
        step = GoodToKnowStep.GoodToKnow2,
        nextStep = OnboardingStep.ReadyToConnect,
        navigator = navigator,
    )
}

@Composable
private fun GoodToKnow(
    step: GoodToKnowStep,
    nextStep: OnboardingStep,
    navigator: Navigator,
) {
    val vm: GoodToKnowViewModel = koinViewModel { parametersOf(step) }
    val state by vm.uiState.collectAsStateWithLifecycle()

    fun finish() {
        vm.complete()
        navigator.navigate(ConnectNavKey, clearBackStack = true)
    }

    GoodToKnowScreen(
        state = state,
        onBackClick =
            if (step == GoodToKnowStep.GoodToKnow1) {
                null
            } else {
                dropUnlessResumed { navigator.goBack() }
            },
        onNextClick = dropUnlessResumed { navigator.navigate(OnboardingNavKey(nextStep)) },
        onSkipClick = dropUnlessResumed { finish() },
    )
}

@Composable
fun GoodToKnowScreen(
    state: GoodToKnowUiState,
    onBackClick: (() -> Unit)?,
    onNextClick: () -> Unit,
    onSkipClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val page = state.page
    OnboardingScaffold(
        modifier = modifier,
        onBackClick = onBackClick,
        onSkipClick = onSkipClick,
        bottomBar = {
            OnboardingBottomBar {
                PrimaryButton(
                    onClick = onNextClick,
                    text = stringResource(id = R.string.next),
                    modifier = Modifier.fillMaxWidth().testTag(ONBOARDING_NEXT_BUTTON_TEST_TAG),
                )
            }
        },
    ) {
        OnboardingInfoIcon()
        OnboardingTitleText(text = stringResource(id = page.title))
        Spacer(Modifier.height(Dimens.smallPadding))
        Image(
            painter = painterResource(page.illustration),
            contentDescription = null,
        )
        OnboardingBodyText(text = stringResource(id = page.body))
    }
}

package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.lib.common.util.then
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_NEXT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Preview
@Composable
private fun PreviewGoodToKnowBlockedContentScreen() {
    AppTheme {
        GoodToKnowScreen(
            step = GoodToKnowStep.GoodToKnow1,
            onBackClick = {},
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
            step = GoodToKnowStep.GoodToKnow2,
            onBackClick = {},
            onNextClick = {},
            onSkipClick = {},
        )
    }
}

@Composable
fun GoodToKnowScreen(
    step: GoodToKnowStep,
    onBackClick: () -> Unit,
    onNextClick: () -> Unit,
    onSkipClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val page = GoodToKnowPage.forStep(step)
    OnboardingScaffold(
        modifier = modifier,
        onBackClick = (step != GoodToKnowStep.GoodToKnow1) then onBackClick,
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
        when (page) {
            GoodToKnowPage.BLOCKED_CONTENT -> SwitchLocationIllustration()
            GoodToKnowPage.EXCLUDING_APPS -> ExcludedAppsIllustration()
        }
        OnboardingBodyText(text = stringResource(id = page.body))
    }
}

@Composable
private fun SwitchLocationIllustration(modifier: Modifier = Modifier) {
    Image(
        modifier = modifier,
        painter = painterResource(R.drawable.onboarding_switch_location),
        contentDescription = null,
    )
}

@Composable
private fun ExcludedAppsIllustration(modifier: Modifier = Modifier) {
    Image(
        modifier = modifier,
        painter = painterResource(R.drawable.onboarding_split_tunneling),
        contentDescription = null,
    )
}

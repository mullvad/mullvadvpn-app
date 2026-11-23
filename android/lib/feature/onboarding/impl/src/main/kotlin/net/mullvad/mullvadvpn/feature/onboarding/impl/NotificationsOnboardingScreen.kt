package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryOutlinedButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_ENABLE_NOTIFICATIONS_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_NO_THANKS_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Preview
@Composable
private fun PreviewNotificationsOnboardingScreen() {
    AppTheme {
        NotificationsOnboardingScreen(
            onEnableNotificationsClick = {},
            onNextClick = {},
        )
    }
}

@Composable
fun NotificationsOnboardingScreen(
    onEnableNotificationsClick: () -> Unit,
    onNextClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    OnboardingScaffold(
        modifier = modifier,
        bottomBar = {
            OnboardingBottomBar {
                PrimaryButton(
                    onClick = onEnableNotificationsClick,
                    text = stringResource(id = R.string.enable_notifications),
                    modifier =
                        Modifier.fillMaxWidth()
                            .testTag(ONBOARDING_ENABLE_NOTIFICATIONS_BUTTON_TEST_TAG),
                )
                PrimaryOutlinedButton(
                    onClick = onNextClick,
                    text = AnnotatedString(stringResource(id = R.string.no_thanks)),
                    modifier =
                        Modifier.fillMaxWidth().testTag(ONBOARDING_NO_THANKS_BUTTON_TEST_TAG),
                )
            }
        },
    ) {
        OnboardingInfoIcon()
        OnboardingTitleText(text = stringResource(id = R.string.want_notifications))
        NotificationPreview(modifier = Modifier.padding(top = Dimens.smallPadding))
        OnboardingBodyText(
            text = stringResource(id = R.string.onboarding_notifications_description),
        )
        OnboardingBodyText(
            text = stringResource(id = R.string.onboarding_notifications_no_ads),
        )
    }
}

@Composable
private fun NotificationPreview(modifier: Modifier = Modifier) {
    Image(
        modifier = modifier,
        painter = painterResource(R.drawable.onboarding_notifications),
        contentDescription = null,
    )
}

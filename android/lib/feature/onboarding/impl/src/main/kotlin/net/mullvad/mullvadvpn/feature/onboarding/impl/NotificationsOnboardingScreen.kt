package net.mullvad.mullvadvpn.feature.onboarding.impl

import android.Manifest
import android.os.Build
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.PermissionStatus
import com.google.accompanist.permissions.rememberPermissionState
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingNavKey
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryOutlinedButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_ENABLE_NOTIFICATIONS_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_NO_THANKS_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel

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

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun NotificationsOnboarding(navigator: Navigator) {
    val vm: NotificationsOnboardingViewModel = koinViewModel()

    fun goToGoodToKnow() {
        vm.onNotificationStepDismissed()
        navigator.navigate(OnboardingNavKey(GoodToKnowStep.GoodToKnow1), clearBackStack = true)
    }

    val notificationPermissionState =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            rememberPermissionState(permission = Manifest.permission.POST_NOTIFICATIONS) {
                goToGoodToKnow()
            }
        } else {
            null
        }

    if (
        notificationPermissionState == null ||
            notificationPermissionState.status == PermissionStatus.Granted
    ) {
        LaunchedEffect(Unit) { goToGoodToKnow() }
    } else {
        NotificationsOnboardingScreen(
            onEnableNotificationsClick =
                dropUnlessResumed { notificationPermissionState.launchPermissionRequest() },
            onNextClick = dropUnlessResumed { goToGoodToKnow() },
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
        NotificationIllustration(modifier = Modifier.padding(top = Dimens.smallPadding))
        OnboardingBodyText(
            text = stringResource(id = R.string.onboarding_notifications_description),
        )
        OnboardingBodyText(
            text = stringResource(id = R.string.onboarding_notifications_no_ads),
        )
    }
}

@Composable
private fun NotificationIllustration(modifier: Modifier = Modifier) {
    Image(
        modifier = modifier,
        painter = painterResource(R.drawable.onboarding_notifications),
        contentDescription = null,
    )
}

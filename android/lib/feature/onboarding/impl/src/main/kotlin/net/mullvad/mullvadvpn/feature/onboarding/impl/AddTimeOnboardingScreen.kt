package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.ui.designsystem.VariantButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_ADD_TIME_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewAddTimeOnboardingScreen() {
    AppTheme { AddTimeOnboardingScreen(onBackClick = {}, onAddTimeClick = {}) }
}

@Composable
fun AddTimeOnboardingScreen(
    onBackClick: () -> Unit,
    onAddTimeClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    OnboardingScaffold(
        modifier = modifier,
        onBackClick = onBackClick,
        bottomBar = {
            OnboardingBottomBar {
                VariantButton(
                    onClick = onAddTimeClick,
                    text = stringResource(id = R.string.add_time),
                    modifier = Modifier.fillMaxWidth().testTag(ONBOARDING_ADD_TIME_BUTTON_TEST_TAG),
                )
            }
        },
    ) {
        Text(
            text = stringResource(id = R.string.add_time),
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.fillMaxWidth(),
        )
        OnboardingBodyText(
            text = stringResource(id = R.string.onboarding_add_time_first_paragraph),
        )
        OnboardingBodyText(
            text = stringResource(id = R.string.onboarding_add_time_second_paragraph),
        )
        OnboardingBodyText(text = stringResource(id = R.string.onboarding_add_time_third_paragraph))
    }
}

package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryOutlinedButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.VariantButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_CONNECT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_LATER_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Preview
@Composable
private fun PreviewReadyToConnectScreen() {
    AppTheme {
        ReadyToConnectScreen(
            selectedLocationTitle = "Sweden",
            onBackClick = {},
            onConnectClick = {},
            onLaterClick = {},
        )
    }
}

@Composable
fun ReadyToConnectScreen(
    selectedLocationTitle: String?,
    onBackClick: () -> Unit,
    onConnectClick: () -> Unit,
    onLaterClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    OnboardingScaffold(
        modifier = modifier,
        verticalArrangement = Arrangement.Center,
        onBackClick = onBackClick,
        bottomBar = {
            OnboardingBottomBar {
                OnboardingBodyText(
                    text = stringResource(id = R.string.you_can_change_location_at_any_time),
                )
                VariantButton(
                    onClick = onConnectClick,
                    text = stringResource(id = R.string.connect),
                    modifier = Modifier.fillMaxWidth().testTag(ONBOARDING_CONNECT_BUTTON_TEST_TAG),
                )
                PrimaryOutlinedButton(
                    onClick = onLaterClick,
                    text = AnnotatedString(stringResource(id = R.string.later)),
                    modifier = Modifier.fillMaxWidth().testTag(ONBOARDING_LATER_BUTTON_TEST_TAG),
                )
            }
        },
    ) {
        OnboardingTitleText(
            modifier = Modifier.fillMaxWidth(),
            text = stringResource(id = R.string.ready_to_connect),
        )
        Spacer(Modifier.height(Dimens.verticalSpace))
        Text(
            modifier = Modifier.fillMaxWidth(),
            text = stringResource(id = R.string.selected_location),
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
        OnboardingTitleText(
            modifier = Modifier.fillMaxWidth(),
            text = selectedLocationTitle ?: stringResource(id = R.string.automatic),
        )
    }
}

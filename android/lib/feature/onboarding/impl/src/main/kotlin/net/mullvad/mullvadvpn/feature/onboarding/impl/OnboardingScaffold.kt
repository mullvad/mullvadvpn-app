package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_SKIP_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Composable
fun OnboardingScaffold(
    modifier: Modifier = Modifier,
    verticalArrangement: Arrangement.Vertical = Arrangement.spacedBy(Dimens.verticalSpace),
    onBackClick: (() -> Unit)? = null,
    onSkipClick: (() -> Unit)? = null,
    bottomBar: @Composable () -> Unit,
    content: @Composable ColumnScope.() -> Unit,
) {
    ScaffoldWithSmallTopBar(
        appBarTitle = "",
        modifier = modifier.testTag(ONBOARDING_SCREEN_TEST_TAG),
        navigationIcon = {
            if (onBackClick != null) NavigateBackIconButton(onNavigateBack = onBackClick)
        },
        actions = {
            if (onSkipClick != null) {
                TextButton(
                    onClick = onSkipClick,
                    colors =
                        ButtonDefaults.textButtonColors(
                            contentColor = MaterialTheme.colorScheme.onSurface,
                        ),
                    modifier = Modifier.testTag(ONBOARDING_SKIP_BUTTON_TEST_TAG),
                ) {
                    Text(text = stringResource(id = R.string.skip))
                }
            }
        },
        bottomBar = bottomBar,
    ) { scaffoldModifier ->
        Column(
            modifier =
                scaffoldModifier
                    .verticalScroll(rememberScrollState())
                    .padding(
                        horizontal = Dimens.sideMarginNew,
                        vertical = Dimens.screenTopMargin,
                    ),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = verticalArrangement,
            content = content,
        )
    }
}

@Composable
fun OnboardingBottomBar(content: @Composable ColumnScope.() -> Unit) {
    Column(
        modifier =
            Modifier.fillMaxWidth()
                .windowInsetsPadding(
                    WindowInsets.safeDrawing.only(
                        WindowInsetsSides.Horizontal + WindowInsetsSides.Bottom,
                    ),
                )
                .padding(
                    horizontal = Dimens.sideMarginNew,
                    vertical = Dimens.screenBottomMargin,
                ),
        verticalArrangement = Arrangement.spacedBy(Dimens.buttonSpacing),
        horizontalAlignment = Alignment.CenterHorizontally,
        content = content,
    )
}

@Composable
fun OnboardingInfoIcon() {
    Icon(
        imageVector = Icons.Rounded.Info,
        contentDescription = null,
        modifier = Modifier.size(Dimens.dialogIconHeight),
    )
}

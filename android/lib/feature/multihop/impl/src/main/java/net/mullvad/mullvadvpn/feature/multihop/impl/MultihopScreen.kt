@file:OptIn(ExperimentalSharedTransitionApi::class)

package net.mullvad.mullvadvpn.feature.multihop.impl

import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.ScreenDescription
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.MULTIHOP_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview("Loading|Enabled|Disabled")
@Composable
private fun PreviewMultihopScreen(
    @PreviewParameter(MultihopUiStatePreviewParameterProvider::class)
    state: Lc<Boolean, MultihopUiState>
) {
    AppTheme { MultihopScreen(state = state, onMultihopClick = {}, onBackClick = {}) }
}

@Composable
fun SharedTransitionScope.Multihop(
    isModal: Boolean,
    navigator: Navigator,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    val viewModel = koinViewModel<MultihopViewModel>() { parametersOf(isModal) }
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    MultihopScreen(
        state = state,
        modifier =
            Modifier.testTag(MULTIHOP_SCREEN_TEST_TAG)
                .sharedBounds(
                    rememberSharedContentState(key = FeatureIndicator.MULTIHOP),
                    animatedVisibilityScope = animatedVisibilityScope,
                ),
        onMultihopClick = viewModel::setMultihop,
        onBackClick = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun MultihopScreen(
    state: Lc<Boolean, MultihopUiState>,
    onMultihopClick: (enable: Boolean) -> Unit,
    onBackClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ScaffoldWithMediumTopBar(
        modifier = modifier,
        appBarTitle = stringResource(id = R.string.multihop),
        navigationIcon = {
            if (state.isModal()) {
                NavigateCloseIconButton(onBackClick)
            } else {
                unlessIsDetail { NavigateBackIconButton(onNavigateBack = onBackClick) }
            }
        },
    ) { modifier ->
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = modifier.padding(horizontal = Dimens.sideMarginNew),
        ) {
            when (state) {
                is Lc.Loading -> Loading()
                is Lc.Content -> {
                    MultihopContent(state = state.value, onMultihopClick = onMultihopClick)
                }
            }
        }
    }
}

@Composable
private fun ColumnScope.MultihopContent(
    state: MultihopUiState,
    onMultihopClick: (enable: Boolean) -> Unit,
) {
    // Scale image to fit width up to certain width
    Image(
        contentScale = ContentScale.FillWidth,
        modifier =
            Modifier.widthIn(max = Dimens.settingsDetailsImageMaxWidth)
                .fillMaxWidth()
                .align(Alignment.CenterHorizontally),
        painter = painterResource(id = R.drawable.multihop_illustration),
        contentDescription = stringResource(R.string.multihop),
    )
    Description()
    SwitchListItem(
        title = stringResource(R.string.enable),
        isToggled = state.enable,
        onCellClicked = onMultihopClick,
    )
}

@Composable
private fun Description() {
    ScreenDescription(
        modifier = Modifier.padding(vertical = Dimens.mediumPadding),
        text = stringResource(R.string.multihop_description),
    )
}

@Composable
private fun Loading() {
    MullvadCircularProgressIndicatorLarge()
}

private fun Lc<Boolean, MultihopUiState>.isModal(): Boolean =
    when (this) {
        is Lc.Loading -> this.value
        is Lc.Content -> this.value.isModal
    }

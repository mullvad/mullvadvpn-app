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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
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
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.multihop.api.WhenNeededInfoNavKey
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.ui.component.DividerButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.InfoListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SelectableListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.ScreenDescription
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.MULTIHOP_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview("Loading|Enabled|Disabled")
@Composable
private fun PreviewMultihopScreen(
    @PreviewParameter(MultihopUiStatePreviewParameterProvider::class)
    state: Lc<Boolean, MultihopUiState>
) {
    AppTheme {
        MultihopScreen(
            state = state,
            onMultihopModeSelected = {},
            onWhenNeededInfoClick = {},
            onBackClick = {},
        )
    }
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
        onMultihopModeSelected = viewModel::setMultihopMode,
        onWhenNeededInfoClick = dropUnlessResumed { navigator.navigate(WhenNeededInfoNavKey) },
        onBackClick = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun MultihopScreen(
    state: Lc<Boolean, MultihopUiState>,
    onMultihopModeSelected: (mode: MultihopMode) -> Unit,
    onWhenNeededInfoClick: () -> Unit,
    onBackClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ScaffoldWithSmallTopBar(
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
        val scrollState = rememberScrollState()
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier =
                modifier
                    .drawVerticalScrollbar(
                        state = scrollState,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                    )
                    .verticalScroll(state = scrollState)
                    .padding(horizontal = Dimens.sideMarginNew),
        ) {
            when (state) {
                is Lc.Loading -> Loading()
                is Lc.Content -> {
                    MultihopContent(
                        state = state.value,
                        onMultihopModeSelected = onMultihopModeSelected,
                        onWhenNeededInfoClick = onWhenNeededInfoClick,
                    )
                }
            }
        }
    }
}

@Composable
private fun ColumnScope.MultihopContent(
    state: MultihopUiState,
    onMultihopModeSelected: (mode: MultihopMode) -> Unit,
    onWhenNeededInfoClick: () -> Unit,
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
    MultihopOptionsList(
        state = state,
        onMultihopModeSelected = onMultihopModeSelected,
        onWhenNeededInfoClick = onWhenNeededInfoClick,
    )
}

@Composable
private fun MultihopOptionsList(
    state: MultihopUiState,
    onMultihopModeSelected: (mode: MultihopMode) -> Unit,
    onWhenNeededInfoClick: () -> Unit,
) {
    InfoListItem(
        hierarchy = Hierarchy.Parent,
        position = Position.Top,
        title = stringResource(R.string.mode),
    )
    HorizontalDivider()
    SelectableListItem(
        hierarchy = Hierarchy.Child1,
        position = Position.Middle,
        isSelected = state.mode == MultihopMode.WHEN_NEEDED,
        onClick = { onMultihopModeSelected(MultihopMode.WHEN_NEEDED) },
        title = stringResource(R.string.when_needed),
        trailingContent = {
            DividerButton(onClick = onWhenNeededInfoClick, icon = Icons.Rounded.Info)
        },
    )
    HorizontalDivider()
    SelectableListItem(
        hierarchy = Hierarchy.Child1,
        position = Position.Middle,
        title = stringResource(R.string.always),
        isSelected = state.mode == MultihopMode.ALWAYS,
        onClick = { onMultihopModeSelected(MultihopMode.ALWAYS) },
    )
    HorizontalDivider()
    SelectableListItem(
        hierarchy = Hierarchy.Child1,
        position = Position.Bottom,
        title = stringResource(R.string.never),
        isSelected = state.mode == MultihopMode.NEVER,
        onClick = { onMultihopModeSelected(MultihopMode.NEVER) },
    )
}

@Composable
private fun Description() {
    ScreenDescription(
        modifier = Modifier.padding(top = Dimens.mediumPadding, bottom = Dimens.largeSpacer),
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

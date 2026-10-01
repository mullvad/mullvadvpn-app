@file:OptIn(ExperimentalSharedTransitionApi::class)

package net.mullvad.mullvadvpn.feature.multihop.impl

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.pager.rememberPagerState
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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.multihop.api.WhenNeededInfoNavKey
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.compose.DescribedIcon
import net.mullvad.mullvadvpn.lib.common.compose.stringResourceWithIcons
import net.mullvad.mullvadvpn.lib.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.ui.component.Carousel
import net.mullvad.mullvadvpn.lib.ui.component.CarouselPage
import net.mullvad.mullvadvpn.lib.ui.component.CarouselParagraph
import net.mullvad.mullvadvpn.lib.ui.component.DividerButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.annotatedStringResource
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.InfoListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SelectableListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.FirstBaselineAlignedIconAndText
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.icon.MultihopWhenNeeded
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.MULTIHOP_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.ui.util.applyIfNotNull
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
    selectedFeature: FeatureIndicator? = null,
    navigator: Navigator,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    val viewModel = koinViewModel<MultihopViewModel>() { parametersOf(isModal) }
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    MultihopScreen(
        state = state,
        modifier =
            Modifier.testTag(MULTIHOP_SCREEN_TEST_TAG).applyIfNotNull(selectedFeature) { feature ->
                Modifier.sharedBounds(
                    rememberSharedContentState(key = feature),
                    animatedVisibilityScope = animatedVisibilityScope,
                )
            },
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
    ) { contentModifier ->
        val scrollState = rememberScrollState()
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier =
                contentModifier
                    .drawVerticalScrollbar(
                        state = scrollState,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                    )
                    .verticalScroll(state = scrollState),
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
private fun MultihopContent(
    state: MultihopUiState,
    onMultihopModeSelected: (mode: MultihopMode) -> Unit,
    onWhenNeededInfoClick: () -> Unit,
) {
    AnimatedVisibility(visible = state.showExtraWhenNeededInfo) {
        FirstBaselineAlignedIconAndText(
            modifier =
                Modifier.padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    bottom = Dimens.mediumPadding,
                ),
            text = stringResource(R.string.automatic_entry_extra_info),
            textStyle = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.Bold),
            icon = MultihopWhenNeeded,
            iconSize = Dimens.smallishIconSize,
            iconTint = MaterialTheme.colorScheme.onSurfaceVariant,
            iconToTextPadding = Dimens.smallSpacer,
            shiftIconDown = true,
        )
    }
    val pagerState = rememberPagerState(pageCount = { MultihopPages.size })
    Carousel(pagerState = pagerState, pages = MultihopPages)
    MultihopOptionsList(
        modifier = Modifier.padding(horizontal = Dimens.smallSpacer),
        state = state,
        onMultihopModeSelected = onMultihopModeSelected,
        onWhenNeededInfoClick = onWhenNeededInfoClick,
    )
}

@Composable
private fun MultihopOptionsList(
    modifier: Modifier = Modifier,
    state: MultihopUiState,
    onMultihopModeSelected: (mode: MultihopMode) -> Unit,
    onWhenNeededInfoClick: () -> Unit,
) {
    Column(modifier) {
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

private val MultihopPages =
    listOf(
        CarouselPage(
            headerImage = R.drawable.multihop_slide_1_main,
            headerImageContentDescription = null,
            paragraphs =
                listOf(
                    @Composable {
                        CarouselParagraph(
                            annotatedStringResource(
                                R.string.multihop_description_slide_1_first_paragraph
                            )
                        )
                    }
                ),
        ),
        CarouselPage(
            headerImage = R.drawable.multihop_slide_2_when_needed,
            headerImageContentDescription = null,
            paragraphs =
                listOf(
                    @Composable {
                        CarouselParagraph(
                            annotatedStringResource(
                                R.string.multihop_description_slide_2_first_paragraph
                            )
                        )
                    },
                    @Composable {
                        CarouselParagraph(
                            annotatedStringResource(
                                R.string.multihop_description_slide_2_second_paragraph
                            )
                        )
                    },
                    @Composable {
                        val iconString =
                            stringResourceWithIcons(
                                id = R.string.multihop_description_slide_2_third_paragraph,
                                DescribedIcon(
                                    icon = MultihopWhenNeeded,
                                    contentDescription =
                                        stringResource(R.string.multihop_when_needed),
                                ),
                                iconTint = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        CarouselParagraph(
                            text = iconString.text,
                            inlineContent = iconString.inlineContent,
                        )
                    },
                ),
        ),
        CarouselPage(
            headerImage = R.drawable.multihop_slide_3_always,
            headerImageContentDescription = null,
            paragraphs =
                listOf(
                    @Composable {
                        CarouselParagraph(
                            annotatedStringResource(
                                R.string.multihop_description_slide_3_first_paragraph
                            )
                        )
                    }
                ),
        ),
        CarouselPage(
            headerImage = R.drawable.multihop_slide_4_never,
            headerImageContentDescription = null,
            paragraphs =
                listOf(
                    @Composable {
                        CarouselParagraph(
                            annotatedStringResource(
                                R.string.multihop_description_slide_4_first_paragraph
                            )
                        )
                    }
                ),
        ),
    )

package net.mullvad.mullvadvpn.feature.daita.impl

import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerState
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.daita.api.DaitaDirectOnlyConfirmationNavKey
import net.mullvad.mullvadvpn.feature.daita.api.DaitaDirectOnlyConfirmedNavResult
import net.mullvad.mullvadvpn.feature.daita.api.DaitaDirectOnlyInfoNavKey
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.ScreenDescription
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.tag.DAITA_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview("Loading|Disabled|Enabled")
@Composable
private fun PreviewDaitaScreen(
    @PreviewParameter(DaitaUiStatePreviewParameterProvider::class) state: Lc<Boolean, DaitaUiState>
) {
    AppTheme {
        DaitaScreen(
            state = state,
            onDaitaEnabled = { _ -> },
            onDirectOnlyClick = { _ -> },
            onDirectOnlyInfoClick = {},
            onBackClick = {},
        )
    }
}

@Composable
fun SharedTransitionScope.Daita(
    navigator: Navigator,
    isModal: Boolean,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    val viewModel = koinViewModel<DaitaViewModel> { parametersOf(isModal) }
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LocalResultStore.current.consumeResult<DaitaDirectOnlyConfirmedNavResult> {
        viewModel.setDirectOnly(true)
    }

    DaitaScreen(
        state = state,
        modifier =
            Modifier.testTag(DAITA_SCREEN_TEST_TAG)
                .sharedBounds(
                    rememberSharedContentState(key = FeatureIndicator.DAITA),
                    animatedVisibilityScope = animatedVisibilityScope,
                ),
        onDaitaEnabled = viewModel::setDaita,
        onDirectOnlyClick = { enable ->
            if (enable) {
                navigator.navigate(DaitaDirectOnlyConfirmationNavKey)
            } else {
                viewModel.setDirectOnly(false)
            }
        },
        onDirectOnlyInfoClick = dropUnlessResumed { navigator.navigate(DaitaDirectOnlyInfoNavKey) },
        onBackClick = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun DaitaScreen(
    state: Lc<Boolean, DaitaUiState>,
    onDaitaEnabled: (enable: Boolean) -> Unit,
    onDirectOnlyClick: (enable: Boolean) -> Unit,
    onDirectOnlyInfoClick: () -> Unit,
    onBackClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(id = R.string.daita),
        modifier = modifier,
        navigationIcon = {
            if (state.isModal()) {
                NavigateCloseIconButton { onBackClick() }
            } else {
                unlessIsDetail { NavigateBackIconButton { onBackClick() } }
            }
        },
    ) { modifier ->
        Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = modifier) {
            when (state) {
                is Lc.Loading -> {
                    Loading()
                }
                is Lc.Content -> {
                    DaitaContent(
                        state = state.value,
                        onDaitaEnabled = onDaitaEnabled,
                        onDirectOnlyClick = onDirectOnlyClick,
                        onDirectOnlyInfoClick = onDirectOnlyInfoClick,
                    )
                }
            }
        }
    }
}

@Composable
private fun DaitaContent(
    state: DaitaUiState,
    onDaitaEnabled: (enable: Boolean) -> Unit,
    onDirectOnlyClick: (enable: Boolean) -> Unit,
    onDirectOnlyInfoClick: () -> Unit,
) {
    val pagerState = rememberPagerState(pageCount = { DaitaPages.entries.size })
    DescriptionPager(pagerState = pagerState)
    PageIndicator(pagerState = pagerState)
    SwitchListItem(
        title = stringResource(R.string.enable),
        isToggled = state.daitaEnabled,
        onCellClicked = onDaitaEnabled,
        position = Position.Top,
        modifier = Modifier.padding(horizontal = Dimens.sideMarginNew),
    )
    HorizontalDivider()
    SwitchListItem(
        title = stringResource(R.string.direct_only),
        isToggled = state.directOnly,
        isEnabled = state.daitaEnabled,
        onCellClicked = onDirectOnlyClick,
        onInfoClicked = onDirectOnlyInfoClick,
        position = Position.Bottom,
        modifier = Modifier.padding(horizontal = Dimens.sideMarginNew),
    )
}

@Composable
private fun DescriptionPager(pagerState: PagerState) {
    HorizontalPager(
        state = pagerState,
        verticalAlignment = Alignment.Top,
        beyondViewportPageCount = DaitaPages.entries.size,
    ) { page ->
        Column(modifier = Modifier.fillMaxWidth()) {
            val page = DaitaPages.entries[page]
            // Scale image to fit width up to certain width
            Image(
                contentScale = ContentScale.FillWidth,
                modifier =
                    Modifier.widthIn(max = Dimens.settingsDetailsImageMaxWidth)
                        .fillMaxWidth()
                        .padding(horizontal = Dimens.sideMarginNew)
                        .align(Alignment.CenterHorizontally),
                painter = painterResource(id = page.image),
                contentDescription = stringResource(R.string.daita),
            )
            DescriptionText(
                firstParagraph = page.textFirstParagraph(),
                secondParagraph = page.textSecondParagraph(),
                thirdParagraph = page.textThirdParagraph(),
            )
        }
    }
}

@Composable
private fun DescriptionText(
    firstParagraph: String,
    secondParagraph: String,
    thirdParagraph: String,
) {
    ScreenDescription(
        modifier =
            Modifier.padding(vertical = Dimens.smallPadding, horizontal = Dimens.sideMarginNew),
        text =
            buildString {
                appendLine(firstParagraph)
                appendLine()
                appendLine(secondParagraph)
                appendLine()
                append(thirdParagraph)
            },
    )
}

@Composable
private fun PageIndicator(pagerState: PagerState) {
    Row(
        Modifier.wrapContentHeight().fillMaxWidth().padding(bottom = Dimens.mediumPadding),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.Bottom,
    ) {
        repeat(pagerState.pageCount) { iteration ->
            val color =
                if (pagerState.currentPage == iteration) MaterialTheme.colorScheme.onPrimary
                else MaterialTheme.colorScheme.primary
            Box(
                modifier =
                    Modifier.padding(Dimens.indicatorPadding)
                        .clip(CircleShape)
                        .background(color)
                        .size(Dimens.indicatorSize)
            )
        }
    }
}

@Composable
private fun Loading() {
    MullvadCircularProgressIndicatorLarge()
}

private fun Lc<Boolean, DaitaUiState>.isModal() =
    when (this) {
        is Lc.Loading -> this.value
        is Lc.Content -> this.value.isModal
    }

private enum class DaitaPages(
    val image: Int,
    val textFirstParagraph: @Composable () -> String,
    val textSecondParagraph: @Composable () -> String,
    val textThirdParagraph: @Composable () -> String,
) {
    FIRST(
        image = R.drawable.daita_illustration_1,
        textFirstParagraph =
            @Composable { stringResource(R.string.daita_description_slide_1_first_paragraph) },
        textSecondParagraph =
            @Composable {
                stringResource(
                    R.string.daita_description_slide_1_second_paragraph,
                    stringResource(id = R.string.daita),
                    stringResource(id = R.string.daita_full),
                )
            },
        textThirdParagraph =
            @Composable { stringResource(R.string.daita_description_slide_1_third_paragraph) },
    ),
    SECOND(
        image = R.drawable.daita_illustration_2,
        textFirstParagraph =
            @Composable {
                stringResource(
                    R.string.daita_description_slide_2_first_paragraph,
                    stringResource(id = R.string.daita),
                )
            },
        textSecondParagraph =
            @Composable {
                stringResource(
                    R.string.daita_description_slide_2_second_paragraph,
                    stringResource(id = R.string.daita),
                )
            },
        textThirdParagraph =
            @Composable {
                stringResource(
                    R.string.daita_description_slide_2_third_paragraph,
                    stringResource(id = R.string.daita),
                )
            },
    ),
}

package net.mullvad.mullvadvpn.feature.daita.impl

import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.ui.component.Carousel
import net.mullvad.mullvadvpn.lib.ui.component.CarouselPage
import net.mullvad.mullvadvpn.lib.ui.component.CarouselParagraph
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.annotatedStringResource
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.toAnnotatedString
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.tag.DAITA_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
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

    DaitaScreen(
        state = state,
        modifier =
            Modifier.testTag(DAITA_SCREEN_TEST_TAG)
                .sharedBounds(
                    rememberSharedContentState(key = FeatureIndicator.DAITA),
                    animatedVisibilityScope = animatedVisibilityScope,
                ),
        onDaitaEnabled = viewModel::setDaita,
        onBackClick = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun DaitaScreen(
    state: Lc<Boolean, DaitaUiState>,
    onDaitaEnabled: (enable: Boolean) -> Unit,
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
                is Lc.Loading -> {
                    Loading()
                }
                is Lc.Content -> {
                    DaitaContent(
                        state = state.value,
                        onDaitaEnabled = onDaitaEnabled,
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
) {
    val pagerState = rememberPagerState(pageCount = { DaitaPages.size })
    Carousel(pagerState = pagerState, pages = DaitaPages)
    SwitchListItem(
        title = stringResource(R.string.enable),
        isToggled = state.daitaEnabled,
        onCellClicked = onDaitaEnabled,
        modifier = Modifier.padding(horizontal = Dimens.sideMarginNew),
    )
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

private val DaitaPages =
    listOf(
        CarouselPage(
            headerImage = R.drawable.daita_illustration_1,
            headerImageContentDescription = null,
            paragraphs =
                listOf(
                    @Composable {
                        CarouselParagraph(
                            annotatedStringResource(
                                R.string.daita_description_slide_1_first_paragraph
                            )
                        )
                    },
                    @Composable {
                        CarouselParagraph(
                            stringResource(
                                    R.string.daita_description_slide_1_second_paragraph,
                                    stringResource(id = R.string.daita),
                                    stringResource(id = R.string.daita_full),
                                )
                                .toAnnotatedString()
                        )
                    },
                    @Composable {
                        CarouselParagraph(
                            annotatedStringResource(
                                R.string.daita_description_slide_1_third_paragraph
                            )
                        )
                    },
                ),
        ),
        CarouselPage(
            headerImage = R.drawable.daita_illustration_2,
            headerImageContentDescription = null,
            paragraphs =
                listOf(
                    @Composable {
                        CarouselParagraph(
                            stringResource(
                                    R.string.daita_description_slide_2_first_paragraph,
                                    stringResource(id = R.string.daita),
                                )
                                .toAnnotatedString()
                        )
                    },
                    @Composable {
                        CarouselParagraph(
                            stringResource(
                                    R.string.daita_description_slide_2_second_paragraph,
                                    stringResource(id = R.string.daita),
                                )
                                .toAnnotatedString()
                        )
                    },
                ),
        ),
    )

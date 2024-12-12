package net.mullvad.mullvadvpn.compose.screen

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
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.DaitaDirectOnlyConfirmationDestination
import com.ramcosta.composedestinations.generated.destinations.DaitaDirectOnlyInfoDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.HeaderSwitchComposeCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.state.DaitaUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.DaitaViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewDaitaScreen() {
    AppTheme { DaitaScreen(state = DaitaUiState(daitaEnabled = false, directOnly = false)) }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun Daita(
    navigator: DestinationsNavigator,
    daitaConfirmationDialogResult: ResultRecipient<DaitaDirectOnlyConfirmationDestination, Boolean>,
) {
    val viewModel = koinViewModel<DaitaViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    daitaConfirmationDialogResult.OnNavResultValue {
        if (it) {
            viewModel.setDirectOnly(true)
        }
    }

    DaitaScreen(
        state = state,
        onDaitaEnabled = viewModel::setDaita,
        onDirectOnlyClick = { enable ->
            if (enable) {
                navigator.navigate(DaitaDirectOnlyConfirmationDestination)
            } else {
                viewModel.setDirectOnly(false)
            }
        },
        onDirectOnlyInfoClick =
            dropUnlessResumed { navigator.navigate(DaitaDirectOnlyInfoDestination) },
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
fun DaitaScreen(
    state: DaitaUiState,
    onDaitaEnabled: (enable: Boolean) -> Unit = {},
    onDirectOnlyClick: (enable: Boolean) -> Unit = {},
    onDirectOnlyInfoClick: () -> Unit = {},
    onBackClick: () -> Unit = {},
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.daita),
        navigationIcon = { NavigateBackIconButton { onBackClick() } },
    ) { modifier ->
        Column(modifier = modifier) {
            val pagerState = rememberPagerState(pageCount = { DaitaPages.entries.size })
            DescriptionPager(pagerState = pagerState)
            PageIndicator(pagerState = pagerState)
            HeaderSwitchComposeCell(
                title = stringResource(R.string.enable),
                isToggled = state.daitaEnabled,
                onCellClicked = onDaitaEnabled,
            )
            HorizontalDivider()
            HeaderSwitchComposeCell(
                title = stringResource(R.string.direct_only),
                isToggled = state.directOnly,
                isEnabled = state.daitaEnabled,
                onCellClicked = onDirectOnlyClick,
                onInfoClicked = onDirectOnlyInfoClick,
            )
        }
    }
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
                        .padding(horizontal = Dimens.mediumPadding)
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
    SwitchComposeSubtitleCell(
        modifier = Modifier.padding(vertical = Dimens.smallPadding),
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

private enum class DaitaPages(
    val image: Int,
    val textFirstParagraph: @Composable () -> String,
    val textSecondParagraph: @Composable () -> String,
    val textThirdParagraph: @Composable () -> String,
) {
    FIRST(
        image = R.drawable.daita_illustration_1,
        textFirstParagraph =
            @Composable {
                stringResource(
                    R.string.daita_description_slide_1_first_paragraph,
                    stringResource(id = R.string.daita),
                    stringResource(id = R.string.daita_full),
                )
            },
        textSecondParagraph =
            @Composable { stringResource(R.string.daita_description_slide_1_second_paragraph) },
        textThirdParagraph =
            @Composable {
                stringResource(
                    R.string.daita_description_slide_1_third_paragraph,
                    stringResource(id = R.string.daita),
                )
            },
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
                    // Duplicated argument to keep compatibility with our common string template
                    // (messages.pot) while also keeping lint happy.
                    stringResource(id = R.string.daita),
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

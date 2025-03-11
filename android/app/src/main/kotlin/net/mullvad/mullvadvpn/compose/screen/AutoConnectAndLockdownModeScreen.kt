package net.mullvad.mullvadvpn.compose.screen

import androidx.annotation.StringRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.PagerState
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ChevronLeft
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.constraintlayout.compose.ConstrainedLayoutReference
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.ConstraintLayoutScope
import androidx.core.text.HtmlCompat
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithLargeTopBarAndButton
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.common.util.openVpnSettings
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInvisible
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.ui.component.toAnnotatedString
import net.mullvad.mullvadvpn.service.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.util.appendHideNavOnPlayBuild

@Preview
@Composable
private fun PreviewAutoConnectAndLockdownModeScreen() {
    AppTheme { AutoConnectAndLockdownModeScreen({}) }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun AutoConnectAndLockdownMode(navigator: DestinationsNavigator) {
    AutoConnectAndLockdownModeScreen(onBackClick = dropUnlessResumed { navigator.navigateUp() })
}

@Composable
fun AutoConnectAndLockdownModeScreen(onBackClick: () -> Unit) {
    val context = LocalContext.current
    ScaffoldWithLargeTopBarAndButton(
        appBarTitle = stringResource(id = R.string.auto_connect_and_lockdown_mode),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        buttonTitle = stringResource(id = R.string.go_to_vpn_settings),
        onButtonClick = { context.openVpnSettings() },
        content = { modifier ->
            Column(modifier = modifier, verticalArrangement = Arrangement.Center) {
                val pagerState = rememberPagerState(pageCount = { PAGES.entries.size })
                val scope = rememberCoroutineScope()
                ConstraintLayout(modifier = Modifier.fillMaxSize()) {
                    val (pager, backButtonRef, nextButtonRef, pageIndicatorRef) = createRefs()

                    AutoConnectCarousel(
                        pagerState = pagerState,
                        backButtonRef = backButtonRef,
                        nextButtonRef = nextButtonRef,
                        pager = pager,
                    )

                    // Go to previous page
                    CarouselNavigationButton(
                        modifier =
                            Modifier.constrainAs(backButtonRef) {
                                top.linkTo(parent.top)
                                start.linkTo(parent.start)
                                bottom.linkTo(parent.bottom)
                            },
                        onClick = {
                            scope.launch {
                                pagerState.animateScrollToPage(pagerState.currentPage - 1)
                            }
                        },
                        isEnabled = { pagerState.currentPage != 0 },
                        imageVector = Icons.Default.ChevronLeft,
                    )

                    // Go to next page
                    CarouselNavigationButton(
                        modifier =
                            Modifier.constrainAs(nextButtonRef) {
                                top.linkTo(parent.top)
                                end.linkTo(parent.end)
                                bottom.linkTo(parent.bottom)
                            },
                        onClick = {
                            scope.launch {
                                pagerState.animateScrollToPage(pagerState.currentPage + 1)
                            }
                        },
                        isEnabled = { pagerState.currentPage != pagerState.pageCount - 1 },
                        imageVector = Icons.Default.ChevronRight,
                    )

                    PageIndicator(
                        pagerState = pagerState,
                        pageIndicatorRef = pageIndicatorRef,
                        pager = pager,
                    )
                }
            }
        },
    )
}

@Composable
private fun ConstraintLayoutScope.AutoConnectCarousel(
    pagerState: PagerState,
    backButtonRef: ConstrainedLayoutReference,
    nextButtonRef: ConstrainedLayoutReference,
    pager: ConstrainedLayoutReference,
) {
    HorizontalPager(
        state = pagerState,
        beyondViewportPageCount = 2,
        modifier =
            Modifier.constrainAs(pager) {
                top.linkTo(parent.top)
                start.linkTo(backButtonRef.end)
                end.linkTo(nextButtonRef.start)
                bottom.linkTo(parent.bottom)
            },
    ) { pageIndex ->
        val page = PAGES.entries[pageIndex]
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier.fillMaxWidth(),
        ) {
            val annotatedTopText = page.annotatedTopText()
            Text(
                modifier = Modifier.padding(horizontal = Dimens.largePadding),
                style =
                    MaterialTheme.typography.titleMedium.copy(
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    ),
                text = annotatedTopText,
            )
            Image(
                modifier = Modifier.padding(top = Dimens.topPadding, bottom = Dimens.bottomPadding),
                painter = painterResource(id = page.image),
                contentDescription = null,
            )
            Text(
                modifier = Modifier.padding(horizontal = Dimens.largePadding),
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                text =
                    HtmlCompat.fromHtml(
                            stringResource(id = page.bottomText),
                            HtmlCompat.FROM_HTML_MODE_COMPACT,
                        )
                        .toAnnotatedString(
                            boldSpanStyle =
                                SpanStyle(
                                    fontWeight = FontWeight.ExtraBold,
                                    color = MaterialTheme.colorScheme.onSurface,
                                )
                        ),
            )
        }
    }
}

@Composable
private fun CarouselNavigationButton(
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
    isEnabled: () -> Boolean,
    imageVector: ImageVector,
) {
    IconButton(
        modifier = modifier.alpha(if (isEnabled.invoke()) AlphaVisible else AlphaInvisible),
        onClick = onClick,
        enabled = isEnabled.invoke(),
    ) {
        Icon(contentDescription = null, imageVector = imageVector)
    }
}

@Composable
private fun ConstraintLayoutScope.PageIndicator(
    pagerState: PagerState,
    pageIndicatorRef: ConstrainedLayoutReference,
    pager: ConstrainedLayoutReference,
) {
    Row(
        Modifier.wrapContentHeight().fillMaxWidth().padding(top = Dimens.topPadding).constrainAs(
            pageIndicatorRef
        ) {
            top.linkTo(pager.bottom)
            end.linkTo(parent.end)
            start.linkTo(parent.start)
        },
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
private fun buildTopText(@StringRes id: Int) = buildAnnotatedString {
    append(stringResource(id = id))
}

@Composable
private fun buildLockdownTopText() = buildAnnotatedString {
    append(buildTopText(id = R.string.auto_connect_carousel_third_slide_top_text))
    append(" ")

    withLink(
        LinkAnnotation.Url(
            stringResource(id = R.string.lockdown_url).appendHideNavOnPlayBuild(IS_PLAY_BUILD)
        )
    ) {
        withStyle(
            style =
                SpanStyle(
                    color = MaterialTheme.colorScheme.onSurface,
                    textDecoration = TextDecoration.Underline,
                )
        ) {
            append(
                stringResource(
                    id = R.string.auto_connect_carousel_third_slide_top_text_website_link
                )
            )
            append(".")
        }
    }
}

private enum class PAGES(
    val annotatedTopText: @Composable () -> AnnotatedString,
    val image: Int,
    val bottomText: Int,
) {
    FIRST(
        annotatedTopText =
            @Composable { buildTopText(id = R.string.auto_connect_carousel_first_slide_top_text) },
        R.drawable.carousel_slide_1_cogwheel,
        R.string.auto_connect_carousel_first_slide_bottom_text,
    ),
    SECOND(
        annotatedTopText =
            @Composable { buildTopText(id = R.string.auto_connect_carousel_second_slide_top_text) },
        R.drawable.carousel_slide_2_always_on,
        R.string.auto_connect_carousel_second_slide_bottom_text,
    ),
    THIRD(
        annotatedTopText = @Composable { buildLockdownTopText() },
        R.drawable.carousel_slide_3_block_connections,
        R.string.auto_connect_carousel_third_slide_bottom_text,
    ),
}

package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.ExperimentalFoundationApi
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
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
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
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.core.text.HtmlCompat
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInvisible
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible

@Preview
@Composable
private fun PreviewAutoConnectCarousel() {
    AppTheme { AutoConnectCarousel() }
}

private enum class PAGES(val topText: Int, val image: Int, val bottomText: Int) {
    FIRST(
        R.string.auto_connect_carousel_first_slide_top_text,
        R.drawable.carousel_slide_1_cogwheel,
        R.string.auto_connect_carousel_first_slide_bottom_text
    ),
    SECOND(
        R.string.auto_connect_carousel_second_slide_top_text,
        R.drawable.carousel_slide_2_always_on,
        R.string.auto_connect_carousel_second_slide_bottom_text
    ),
    THIRD(
        R.string.auto_connect_carousel_third_slide_top_text,
        R.drawable.carousel_slide_3_block_connections,
        R.string.auto_connect_carousel_third_slide_bottom_text
    )
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun AutoConnectCarousel() {
    val pagerState = rememberPagerState(pageCount = { PAGES.entries.size })
    val scope = rememberCoroutineScope()
    ConstraintLayout(
        modifier = Modifier.fillMaxSize(),
    ) {
        val (pager, backButtonRef, nextButtonRef, pageIndicatorRef) = createRefs()
        HorizontalPager(
            state = pagerState,
            beyondBoundsPageCount = 2,
            modifier =
                Modifier.constrainAs(pager) {
                    top.linkTo(parent.top)
                    start.linkTo(backButtonRef.end)
                    end.linkTo(nextButtonRef.start)
                    bottom.linkTo(parent.bottom)
                }
        ) { page ->
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(
                    modifier = Modifier.padding(horizontal = Dimens.largePadding),
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSecondary,
                    text =
                        HtmlCompat.fromHtml(
                                stringResource(id = PAGES.entries[page].topText),
                                HtmlCompat.FROM_HTML_MODE_COMPACT
                            )
                            .toAnnotatedString(
                                boldSpanStyle =
                                    SpanStyle(
                                        fontWeight = FontWeight.ExtraBold,
                                        color = MaterialTheme.colorScheme.onPrimary
                                    )
                            )
                )
                Image(
                    modifier =
                        Modifier.padding(top = Dimens.topPadding, bottom = Dimens.bottomPadding),
                    painter = painterResource(id = PAGES.entries[page].image),
                    contentDescription = null,
                )
                Text(
                    modifier = Modifier.padding(horizontal = Dimens.largePadding),
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSecondary,
                    text =
                        HtmlCompat.fromHtml(
                                stringResource(id = PAGES.entries[page].bottomText),
                                HtmlCompat.FROM_HTML_MODE_COMPACT
                            )
                            .toAnnotatedString(
                                boldSpanStyle =
                                    SpanStyle(
                                        fontWeight = FontWeight.ExtraBold,
                                        color = MaterialTheme.colorScheme.onPrimary
                                    )
                            )
                )
            }
        }

        IconButton(
            modifier =
                Modifier.constrainAs(backButtonRef) {
                        top.linkTo(parent.top)
                        start.linkTo(parent.start)
                        bottom.linkTo(parent.bottom)
                    }
                    .alpha(if (pagerState.currentPage != 0) AlphaVisible else AlphaInvisible),
            onClick = {
                scope.launch { pagerState.animateScrollToPage(pagerState.currentPage - 1) }
            },
            enabled = pagerState.currentPage != 0
        ) {
            Icon(
                painter = painterResource(id = R.drawable.icon_chevron),
                contentDescription = null,
                tint = Color.Unspecified,
                modifier = Modifier.rotate(180f).alpha(AlphaDescription)
            )
        }

        IconButton(
            modifier =
                Modifier.constrainAs(nextButtonRef) {
                        top.linkTo(parent.top)
                        end.linkTo(parent.end)
                        bottom.linkTo(parent.bottom)
                    }
                    .alpha(
                        if (pagerState.currentPage != pagerState.pageCount - 1) AlphaVisible
                        else AlphaInvisible
                    ),
            onClick = {
                scope.launch { pagerState.animateScrollToPage(pagerState.currentPage + 1) }
            },
            enabled = pagerState.currentPage != pagerState.pageCount - 1
        ) {
            Icon(
                painter = painterResource(id = R.drawable.icon_chevron),
                contentDescription = null,
                tint = Color.Unspecified,
                modifier = Modifier.alpha(AlphaDescription)
            )
        }
        Row(
            Modifier.wrapContentHeight()
                .fillMaxWidth()
                .padding(top = Dimens.topPadding)
                .constrainAs(pageIndicatorRef) {
                    top.linkTo(pager.bottom)
                    end.linkTo(parent.end)
                    start.linkTo(parent.start)
                },
            horizontalArrangement = Arrangement.Center,
            verticalAlignment = Alignment.Bottom
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
}

package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
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

@Preview
@Composable
fun PreviewAutoConnectCarousel() {
    AppTheme { AutoConnectCarousel() }
}

private const val CAROUSEL_PAGE_SIZE = 3

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun AutoConnectCarousel() {
    val pagerState = rememberPagerState(pageCount = { CAROUSEL_PAGE_SIZE })
    HorizontalPager(state = pagerState, modifier = Modifier.fillMaxSize()) { page ->
        val scope = rememberCoroutineScope()
        ConstraintLayout(
            modifier = Modifier.fillMaxSize(),
        ) {
            val (
                upperTextRef,
                backButtonRef,
                imageRef,
                nextButtonRef,
                lowerTextRef,
                pageIndicatorRef) =
                createRefs()
            Text(
                modifier =
                    Modifier.padding(horizontal = Dimens.largePadding).constrainAs(upperTextRef) {
                        start.linkTo(parent.start)
                        end.linkTo(parent.end)
                        bottom.linkTo(imageRef.top)
                    },
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSecondary,
                text =
                    HtmlCompat.fromHtml(
                            stringResource(
                                id =
                                    when (page) {
                                        0 -> R.string.auto_connect_carousel_first_slide_top_text
                                        1 -> R.string.auto_connect_carousel_second_slide_top_text
                                        else -> R.string.auto_connect_carousel_third_slide_top_text
                                    }
                            ),
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
            if (page != 0) {
                IconButton(
                    modifier =
                        Modifier.constrainAs(backButtonRef) {
                            top.linkTo(parent.top)
                            start.linkTo(parent.start)
                            bottom.linkTo(parent.bottom)
                        },
                    onClick = {
                        scope.launch { pagerState.scrollToPage(pagerState.currentPage - 1) }
                    },
                ) {
                    Icon(
                        painter = painterResource(id = R.drawable.icon_chevron),
                        contentDescription = null,
                        tint = Color.Unspecified,
                        modifier = Modifier.rotate(180f).alpha(AlphaDescription)
                    )
                }
            }

            Image(
                modifier =
                    Modifier.padding(top = Dimens.topPadding, bottom = Dimens.bottomPadding)
                        .constrainAs(imageRef) {
                            top.linkTo(parent.top)
                            start.linkTo(parent.start)
                            bottom.linkTo(parent.bottom)
                            end.linkTo(parent.end)
                        },
                painter =
                    when (page) {
                        0 -> painterResource(id = R.drawable.carousel_slide_1_cogwheel)
                        1 -> painterResource(id = R.drawable.carousel_slide_2_always_on)
                        else -> painterResource(id = R.drawable.carousel_slide_3_block_connections)
                    },
                contentDescription = null,
            )

            if (page < CAROUSEL_PAGE_SIZE) {
                IconButton(
                    modifier =
                        Modifier.constrainAs(nextButtonRef) {
                            top.linkTo(parent.top)
                            end.linkTo(parent.end)
                            bottom.linkTo(parent.bottom)
                        },
                    onClick = {
                        scope.launch { pagerState.scrollToPage(pagerState.currentPage + 1) }
                    }
                ) {
                    Icon(
                        painter = painterResource(id = R.drawable.icon_chevron),
                        contentDescription = null,
                        tint = Color.Unspecified,
                        modifier = Modifier.size(Dimens.titleIconSize).alpha(AlphaDescription)
                    )
                }
            }
            Text(
                modifier =
                    Modifier.padding(horizontal = Dimens.largePadding).constrainAs(lowerTextRef) {
                        top.linkTo(imageRef.bottom)
                        end.linkTo(parent.end)
                        start.linkTo(parent.start)
                    },
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSecondary,
                text =
                    HtmlCompat.fromHtml(
                            stringResource(
                                id =
                                    when (page) {
                                        0 -> R.string.auto_connect_carousel_first_slide_bottom_text
                                        1 -> R.string.auto_connect_carousel_second_slide_bottom_text
                                        else ->
                                            R.string.auto_connect_carousel_third_slide_bottom_text
                                    }
                            ),
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

            Row(
                Modifier.wrapContentHeight()
                    .fillMaxWidth()
                    .padding(top = Dimens.topPadding)
                    .constrainAs(pageIndicatorRef) {
                        top.linkTo(lowerTextRef.bottom)

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
}

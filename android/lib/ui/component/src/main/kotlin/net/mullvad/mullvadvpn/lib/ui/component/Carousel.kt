package net.mullvad.mullvadvpn.lib.ui.component

import androidx.annotation.DrawableRes
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
import androidx.compose.foundation.text.InlineTextContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.common.compose.joinToAnnotatedString
import net.mullvad.mullvadvpn.lib.ui.component.text.ScreenDescription
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Preview
@Composable
private fun PreviewCarousel() {
    AppTheme {
        val pagerState = rememberPagerState(pageCount = { 2 })
        Carousel(
            pagerState = pagerState,
            pages =
                listOf(
                    CarouselPage(
                        headerImage = R.drawable.daita_illustration_1,
                        headerImageContentDescription = null,
                        paragraphs =
                            listOf(
                                @Composable {
                                    CarouselParagraph(
                                        annotatedStringResource(
                                            R.string.api_access_method_info_first_line
                                        )
                                    )
                                },
                                @Composable {
                                    CarouselParagraph(
                                        annotatedStringResource(
                                            R.string.api_access_method_info_second_line
                                        )
                                    )
                                },
                            ),
                    ),
                    CarouselPage(
                        headerImage = R.drawable.daita_illustration_2,
                        headerImageContentDescription = null,
                        paragraphs =
                            listOf(@Composable { CarouselParagraph("page 2".toAnnotatedString()) }),
                    ),
                ),
        )
    }
}

data class CarouselPage(
    @DrawableRes val headerImage: Int?,
    val headerImageContentDescription: String?,
    val paragraphs: List<@Composable () -> CarouselParagraph>,
)

data class CarouselParagraph(
    val text: AnnotatedString,
    val inlineContent: Map<String, InlineTextContent> = mapOf(),
)

@Composable
fun Carousel(
    modifier: Modifier = Modifier,
    pagerState: PagerState,
    pages: List<CarouselPage>,
) {
    Column(modifier = modifier) {
        HorizontalPager(
            state = pagerState,
            verticalAlignment = Alignment.Top,
            beyondViewportPageCount = minOf(pagerState.pageCount, BeyondViewportPageCount),
        ) { pageIndex ->
            Column(modifier = Modifier.fillMaxWidth()) {
                val page = pages[pageIndex]
                val image = page.headerImage

                if (image != null) {
                    // Scale image to fit width up to certain width
                    Image(
                        contentScale = ContentScale.FillWidth,
                        modifier =
                            Modifier.widthIn(max = Dimens.settingsDetailsImageMaxWidth)
                                .fillMaxWidth()
                                .padding(horizontal = Dimens.sideMarginNew)
                                .align(Alignment.CenterHorizontally),
                        painter = painterResource(image),
                        contentDescription = page.headerImageContentDescription,
                    )
                }
                val paragraphs = page.paragraphs.map { it() }
                ScreenDescription(
                    modifier =
                        Modifier.padding(
                            vertical = Dimens.smallPadding,
                            horizontal = Dimens.sideMarginNew,
                        ),
                    text = paragraphs.map(CarouselParagraph::text).joinToAnnotatedString("\n\n"),
                    inlineContent = buildMap { paragraphs.forEach { putAll(it.inlineContent) } },
                )
            }
        }
        PageIndicator(pagerState)
    }
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

private const val BeyondViewportPageCount = 4

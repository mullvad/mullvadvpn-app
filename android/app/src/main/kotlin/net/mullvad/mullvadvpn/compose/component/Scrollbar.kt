package net.mullvad.mullvadvpn.compose.component

/*
 * Code snippet taken from https://gist.github.com/mxalbert1996/33a360fcab2105a31e5355af98216f5a
 *
 * MIT License
 *
 * Copyright (c) 2022 Albert Chang
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

import android.view.ViewConfiguration
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.util.fastSumBy
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.collectLatest

fun Modifier.drawHorizontalScrollbar(
    state: ScrollState,
    reverseScrolling: Boolean = false
): Modifier = drawScrollbar(state, Orientation.Horizontal, reverseScrolling)

fun Modifier.drawVerticalScrollbar(
    state: ScrollState,
    reverseScrolling: Boolean = false
): Modifier = drawScrollbar(state, Orientation.Vertical, reverseScrolling)

private fun Modifier.drawScrollbar(
    state: ScrollState,
    orientation: Orientation,
    reverseScrolling: Boolean
): Modifier =
    drawScrollbar(orientation, reverseScrolling) { reverseDirection, atEnd, color, alpha ->
        if (state.maxValue > 0) {
            val canvasSize = if (orientation == Orientation.Horizontal) size.width else size.height
            val totalSize = canvasSize + state.maxValue
            val thumbSize = canvasSize / totalSize * canvasSize
            val startOffset = state.value / totalSize * canvasSize
            drawScrollbar(
                orientation,
                reverseDirection,
                atEnd,
                color,
                alpha,
                thumbSize,
                startOffset
            )
        }
    }

fun Modifier.drawHorizontalScrollbar(
    state: LazyListState,
    reverseScrolling: Boolean = false
): Modifier = drawScrollbar(state, Orientation.Horizontal, reverseScrolling)

fun Modifier.drawVerticalScrollbar(
    state: LazyListState,
    reverseScrolling: Boolean = false
): Modifier = drawScrollbar(state, Orientation.Vertical, reverseScrolling)

private fun Modifier.drawScrollbar(
    state: LazyListState,
    orientation: Orientation,
    reverseScrolling: Boolean
): Modifier =
    drawScrollbar(orientation, reverseScrolling) { reverseDirection, atEnd, color, alpha ->
        val layoutInfo = state.layoutInfo
        val viewportSize = layoutInfo.viewportEndOffset - layoutInfo.viewportStartOffset
        val items = layoutInfo.visibleItemsInfo
        val itemsSize = items.fastSumBy { it.size }
        if (items.size < layoutInfo.totalItemsCount || itemsSize > viewportSize) {
            val estimatedItemSize = if (items.isEmpty()) 0f else itemsSize.toFloat() / items.size
            val totalSize = estimatedItemSize * layoutInfo.totalItemsCount
            val canvasSize = if (orientation == Orientation.Horizontal) size.width else size.height
            val thumbSize = viewportSize / totalSize * canvasSize
            val startOffset =
                if (items.isEmpty()) 0f
                else
                    items.first().run {
                        (estimatedItemSize * index - offset) / totalSize * canvasSize
                    }
            drawScrollbar(
                orientation,
                reverseDirection,
                atEnd,
                color,
                alpha,
                thumbSize,
                startOffset
            )
        }
    }

fun Modifier.drawVerticalScrollbar(
    state: LazyGridState,
    spanCount: Int,
    reverseScrolling: Boolean = false
): Modifier =
    drawScrollbar(Orientation.Vertical, reverseScrolling) { reverseDirection, atEnd, color, alpha ->
        val layoutInfo = state.layoutInfo
        val viewportSize = layoutInfo.viewportEndOffset - layoutInfo.viewportStartOffset
        val items = layoutInfo.visibleItemsInfo
        val rowCount = (items.size + spanCount - 1) / spanCount
        var itemsSize = 0
        for (i in 0 until rowCount) {
            itemsSize += items[i * spanCount].size.height
        }
        if (items.size < layoutInfo.totalItemsCount || itemsSize > viewportSize) {
            val estimatedItemSize = if (rowCount == 0) 0f else itemsSize.toFloat() / rowCount
            val totalRow = (layoutInfo.totalItemsCount + spanCount - 1) / spanCount
            val totalSize = estimatedItemSize * totalRow
            val canvasSize = size.height
            val thumbSize = viewportSize / totalSize * canvasSize
            val startOffset =
                if (rowCount == 0) 0f
                else
                    items.first().run {
                        val rowIndex = index / spanCount
                        (estimatedItemSize * rowIndex - offset.y) / totalSize * canvasSize
                    }
            drawScrollbar(
                Orientation.Vertical,
                reverseDirection,
                atEnd,
                color,
                alpha,
                thumbSize,
                startOffset
            )
        }
    }

private fun DrawScope.drawScrollbar(
    orientation: Orientation,
    reverseDirection: Boolean,
    atEnd: Boolean,
    color: Color,
    alpha: () -> Float,
    thumbSize: Float,
    startOffset: Float
) {
    val thicknessPx = Thickness.toPx()
    val topLeft =
        if (orientation == Orientation.Horizontal) {
            Offset(
                if (reverseDirection) size.width - startOffset - thumbSize else startOffset,
                if (atEnd) size.height - thicknessPx else 0f
            )
        } else {
            Offset(
                if (atEnd) size.width - thicknessPx else 0f,
                if (reverseDirection) size.height - startOffset - thumbSize else startOffset
            )
        }
    val size =
        if (orientation == Orientation.Horizontal) {
            Size(thumbSize, thicknessPx)
        } else {
            Size(thicknessPx, thumbSize)
        }

    drawRect(color = color, topLeft = topLeft, size = size, alpha = alpha())
}

private fun Modifier.drawScrollbar(
    orientation: Orientation,
    reverseScrolling: Boolean,
    onDraw:
        DrawScope.(
            reverseDirection: Boolean, atEnd: Boolean, color: Color, alpha: () -> Float
        ) -> Unit
): Modifier = composed {
    val scrolled = remember {
        MutableSharedFlow<Unit>(
            extraBufferCapacity = 1,
            onBufferOverflow = BufferOverflow.DROP_OLDEST
        )
    }
    val nestedScrollConnection =
        remember(orientation, scrolled) {
            object : NestedScrollConnection {
                override fun onPostScroll(
                    consumed: Offset,
                    available: Offset,
                    source: NestedScrollSource
                ): Offset {
                    val delta =
                        if (orientation == Orientation.Horizontal) consumed.x else consumed.y
                    if (delta != 0f) scrolled.tryEmit(Unit)
                    return Offset.Zero
                }
            }
        }

    val alpha = remember { Animatable(0f) }
    LaunchedEffect(scrolled, alpha) {
        scrolled.collectLatest {
            alpha.snapTo(1f)
            delay(ViewConfiguration.getScrollDefaultDelay().toLong())
            alpha.animateTo(0f, animationSpec = FadeOutAnimationSpec)
        }
    }

    val isLtr = LocalLayoutDirection.current == LayoutDirection.Ltr
    val reverseDirection =
        if (orientation == Orientation.Horizontal) {
            if (isLtr) reverseScrolling else !reverseScrolling
        } else reverseScrolling
    val atEnd = if (orientation == Orientation.Vertical) isLtr else true

    val color = BarColor

    Modifier.nestedScroll(nestedScrollConnection).drawWithContent {
        drawContent()
        onDraw(reverseDirection, atEnd, color, alpha::value)
    }
}

private val BarColor: Color
    @Composable get() = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)

private val Thickness = 4.dp
private val FadeOutAnimationSpec =
    tween<Float>(durationMillis = ViewConfiguration.getScrollBarFadeDuration())

@Preview(widthDp = 400, heightDp = 400, showBackground = true)
@Composable
internal fun ScrollbarPreview() {
    val state = rememberScrollState()
    Column(
        modifier = Modifier.drawVerticalScrollbar(state).verticalScroll(state),
    ) {
        repeat(50) {
            Text(text = "Item ${it + 1}", modifier = Modifier.fillMaxWidth().padding(16.dp))
        }
    }
}

@Preview(widthDp = 400, heightDp = 400, showBackground = true)
@Composable
internal fun LazyListScrollbarPreview() {
    val state = rememberLazyListState()
    LazyColumn(modifier = Modifier.drawVerticalScrollbar(state), state = state) {
        items(50) {
            Text(text = "Item ${it + 1}", modifier = Modifier.fillMaxWidth().padding(16.dp))
        }
    }
}

@Preview(widthDp = 400, showBackground = true)
@Composable
internal fun HorizontalScrollbarPreview() {
    val state = rememberScrollState()
    Row(modifier = Modifier.drawHorizontalScrollbar(state).horizontalScroll(state)) {
        repeat(50) {
            Text(
                text = (it + 1).toString(),
                modifier = Modifier.padding(horizontal = 8.dp, vertical = 16.dp)
            )
        }
    }
}

@Preview(widthDp = 400, showBackground = true)
@Composable
internal fun LazyListHorizontalScrollbarPreview() {
    val state = rememberLazyListState()
    LazyRow(modifier = Modifier.drawHorizontalScrollbar(state), state = state) {
        items(50) {
            Text(
                text = (it + 1).toString(),
                modifier = Modifier.padding(horizontal = 8.dp, vertical = 16.dp)
            )
        }
    }
}

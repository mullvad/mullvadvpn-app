package net.mullvad.mullvadvpn.compose.extensions

import androidx.compose.foundation.gestures.animateScrollBy
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.HorizontalDivider
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

inline fun LazyListScope.itemWithDivider(
    key: Any? = null,
    contentType: Any? = null,
    crossinline itemContent: @Composable LazyItemScope.() -> Unit,
) =
    item(key = key, contentType = contentType) {
        itemContent()
        HorizontalDivider(color = Color.Transparent)
    }

inline fun <T> LazyListScope.itemsWithDivider(
    items: List<T>,
    noinline key: ((item: T) -> Any)? = null,
    noinline contentType: (item: T) -> Any? = { null },
    crossinline itemContent: @Composable LazyItemScope.(item: T) -> Unit,
) =
    items(items = items, key = key, contentType = contentType) { item ->
        itemContent(item)
        HorizontalDivider(color = Color.Transparent)
    }

inline fun <T> LazyListScope.itemsIndexedWithDivider(
    items: List<T>,
    noinline key: ((index: Int, item: T) -> Any)? = null,
    crossinline contentType: (index: Int, item: T) -> Any? = { _, _ -> null },
    crossinline itemContent: @Composable LazyItemScope.(index: Int, item: T) -> Unit,
) =
    itemsIndexed(items = items, key = key, contentType = contentType) { index, item ->
        itemContent(index, item)
        HorizontalDivider(color = Color.Transparent)
    }

suspend fun LazyListState.animateScrollAndCentralizeItem(index: Int) {
    val itemInfo = this.layoutInfo.visibleItemsInfo.firstOrNull { it.index == index }
    if (itemInfo != null) {
        val center = layoutInfo.viewportEndOffset / 2
        val childCenter = itemInfo.offset + itemInfo.size / 2
        animateScrollBy((childCenter - center).toFloat())
    } else {
        animateScrollToItem(index)
    }
}

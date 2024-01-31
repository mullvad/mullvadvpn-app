package net.mullvad.mullvadvpn.compose.extensions

import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.Divider
import androidx.compose.runtime.Composable

inline fun LazyListScope.itemWithDivider(
    key: Any? = null,
    contentType: Any? = null,
    crossinline itemContent: @Composable LazyItemScope.() -> Unit
) =
    item(key = key, contentType = contentType) {
        itemContent()
        Divider()
    }

inline fun <T> LazyListScope.itemsIndexedWithDivider(
    items: List<T>,
    noinline key: ((index: Int, item: T) -> Any)? = null,
    crossinline contentType: (index: Int, item: T) -> Any? = { _, _ -> null },
    crossinline itemContent: @Composable LazyItemScope.(index: Int, item: T) -> Unit
) =
    itemsIndexed(items = items, key = key, contentType = contentType) { index, item ->
        itemContent(index, item)
        Divider()
    }

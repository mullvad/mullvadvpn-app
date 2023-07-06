package net.mullvad.mullvadvpn.compose.extensions

import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
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

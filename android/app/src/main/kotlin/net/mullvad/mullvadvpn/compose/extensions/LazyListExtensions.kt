package net.mullvad.mullvadvpn.compose.extensions

import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material.Divider
import androidx.compose.runtime.Composable

inline fun LazyListScope.itemWithDivider(
    crossinline itemContent: @Composable LazyItemScope.() -> Unit
) = item {
    itemContent()
    Divider()
}

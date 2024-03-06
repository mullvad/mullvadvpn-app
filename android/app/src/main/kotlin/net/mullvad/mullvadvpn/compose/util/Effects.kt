package net.mullvad.mullvadvpn.compose.util

import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

@Composable
fun RunOnKeyChange(key: Any, block: suspend CoroutineScope.() -> Unit) {
    val scope = rememberCoroutineScope()
    rememberSaveable(key) {
        scope.launch { block() }
        key
    }
}

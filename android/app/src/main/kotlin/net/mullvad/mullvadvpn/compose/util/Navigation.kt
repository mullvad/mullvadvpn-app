package net.mullvad.mullvadvpn.compose.util

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisallowComposableCalls
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import com.ramcosta.composedestinations.spec.DestinationSpec

@Composable
fun <D : DestinationSpec<*>, V> ResultRecipient<D, V>.OnNavResultValue(
    onValue: @DisallowComposableCalls (value: V) -> Unit
) = onNavResult {
    when (it) {
        NavResult.Canceled -> Unit
        is NavResult.Value -> onValue(it.value)
    }
}

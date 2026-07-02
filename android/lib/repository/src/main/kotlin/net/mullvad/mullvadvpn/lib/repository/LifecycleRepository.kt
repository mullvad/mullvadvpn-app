package net.mullvad.mullvadvpn.lib.repository

import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.LifecycleOwner
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

class LifecycleRepository : LifecycleEventObserver {
    private val _lifecycleFlow = MutableStateFlow(Lifecycle.State.INITIALIZED)
    val lifecycleFlow = _lifecycleFlow.asStateFlow()

    override fun onStateChanged(source: LifecycleOwner, event: Lifecycle.Event) {
        _lifecycleFlow.value = event.targetState
    }
}

package net.mullvad.mullvadvpn.lib.intent

import android.content.Intent
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow

class IntentProvider {
    private val _intents = MutableStateFlow<Intent?>(null)
    val intents: Flow<Intent?> = _intents

    fun setStartIntent(intent: Intent?) {
        _intents.tryEmit(intent)
    }

    fun getLatestIntent(): Intent? = _intents.value
}

package net.mullvad.mullvadvpn.feature.home.impl.connect

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

class HasShowNotificationPromptRepository {

    private val _hasShowNotificationPrompt = MutableStateFlow(false)
    val hasShowNotificationPrompt: StateFlow<Boolean> = _hasShowNotificationPrompt

    fun setHasShowNotificationPrompt(value: Boolean) {
        _hasShowNotificationPrompt.value = value
    }
}

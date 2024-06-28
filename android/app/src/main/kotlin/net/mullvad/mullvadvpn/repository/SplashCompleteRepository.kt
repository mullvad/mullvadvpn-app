package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.flow.MutableStateFlow

class SplashCompleteRepository {
    private val _splashComplete = MutableStateFlow(false)

    fun isSplashComplete() = _splashComplete.value

    fun onSplashCompleted() {
        _splashComplete.value = true
    }
}

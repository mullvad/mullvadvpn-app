package net.mullvad.mullvadvpn.service.notifications

import kotlinx.coroutines.flow.StateFlow

interface ShouldBeOnForegroundProvider {
    val shouldBeOnForeground: StateFlow<Boolean>

    fun startForeground() {}

    fun stopForeground() {}
}

package net.mullvad.mullvadvpn.lib.pushnotification

import kotlinx.coroutines.flow.Flow

interface ShouldBeOnForegroundProvider {
    val shouldBeOnForeground: Flow<Boolean>
}

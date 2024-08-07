package net.mullvad.mullvadvpn.service.notifications

import kotlinx.coroutines.flow.Flow

interface ShouldBeOnForegroundProvider {
    val shouldBeOnForeground: Flow<Boolean>
}

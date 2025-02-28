package net.mullvad.mullvadvpn.widget

import kotlinx.coroutines.MainScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy

class WidgetProvider(
    private val connectionProxy: ConnectionProxy
) {
    fun state() = connectionProxy.tunnelState.stateIn(
        scope = MainScope(),
        started = SharingStarted.Lazily,
        initialValue = TunnelState.Disconnected(null)
    )
}

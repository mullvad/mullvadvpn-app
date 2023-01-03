package net.mullvad.mullvadvpn.ui.notification

import android.content.Context
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.util.getErrorNotificationResources
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorState

class TunnelStateNotification(
    private val context: Context,
) : InAppNotification() {
    init {
        status = StatusLevel.Error
        onClick = null
        showIcon = false
    }

    fun updateTunnelState(state: TunnelState) {
        when (state) {
            is TunnelState.Disconnecting -> {
                when (state.actionAfterDisconnect) {
                    ActionAfterDisconnect.Nothing -> hide()
                    ActionAfterDisconnect.Block -> showGenericBlockingMessage()
                    ActionAfterDisconnect.Reconnect -> showGenericBlockingMessage()
                }
            }
            is TunnelState.Disconnected -> hide()
            is TunnelState.Connecting -> showGenericBlockingMessage()
            is TunnelState.Connected -> hide()
            is TunnelState.Error -> showError(state.errorState)
        }

        update()
    }

    private fun showError(error: ErrorState) {
        // if the error state is null, we can assume that we are secure
        error.getErrorNotificationResources(context).apply {
            title = this.getTitleText(context.resources)
            message = this.getMessageText(context.resources)
        }
        shouldShow = true
    }

    private fun showGenericBlockingMessage() {
        title = context.getString(R.string.blocking_all_connections)
        message = null
        shouldShow = true
    }

    private fun hide() {
        shouldShow = false
    }
}

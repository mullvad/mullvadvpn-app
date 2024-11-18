package net.mullvad.mullvadvpn.lib.model

import android.content.Intent

sealed interface NotificationAction {

    sealed interface AccountExpiry : NotificationAction {
        data object Open : AccountExpiry
    }

    sealed interface Tunnel : NotificationAction {
        data object Connect : Tunnel

        data object Disconnect : Tunnel

        data object Cancel : Tunnel

        data object Dismiss : Tunnel

        data class RequestPermission(val prepareIntent: Intent) : Tunnel
    }
}

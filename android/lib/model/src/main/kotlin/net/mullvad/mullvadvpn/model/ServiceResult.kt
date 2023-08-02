package net.mullvad.mullvadvpn.model

import android.os.IBinder

data class ServiceResult(val binder: IBinder?) {
    enum class ConnectionState {
        CONNECTED,
        DISCONNECTED
    }

    val connectionState: ConnectionState
        get() {
            return if (binder == null) {
                ConnectionState.DISCONNECTED
            } else {
                ConnectionState.CONNECTED
            }
        }

    companion object {
        val NOT_CONNECTED = ServiceResult(null)
    }
}

package net.mullvad.mullvadvpn.model

sealed class ActionAfterDisconnect {
    class Nothing : ActionAfterDisconnect()
    class Block : ActionAfterDisconnect()
    class Reconnect : ActionAfterDisconnect()
}

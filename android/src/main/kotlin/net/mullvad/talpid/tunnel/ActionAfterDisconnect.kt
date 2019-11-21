package net.mullvad.talpid.tunnel

sealed class ActionAfterDisconnect {
    class Nothing : ActionAfterDisconnect()
    class Block : ActionAfterDisconnect()
    class Reconnect : ActionAfterDisconnect()
}

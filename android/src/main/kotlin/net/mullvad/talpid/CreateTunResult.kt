package net.mullvad.talpid

sealed class CreateTunResult {
    open val isOpen
        get() = false

    class Success(val tunFd: Int) : CreateTunResult() {
        override val isOpen
            get() = true
    }

    class PermissionDenied : CreateTunResult()
    class TunnelDeviceError : CreateTunResult()
}

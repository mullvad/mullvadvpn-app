package net.mullvad.talpid

sealed class CreateTunResult {
    class Success(val tunFd: Int) : CreateTunResult()
    class PermissionDenied : CreateTunResult()
    class TunnelDeviceError : CreateTunResult()
}

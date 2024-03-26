package net.mullvad.mullvadvpn.ui.serviceconnection

fun ServiceConnectionManager.authTokenCache() =
    this.connectionState.value.readyContainer()?.authTokenCache

fun ServiceConnectionManager.deviceDataSource() =
    this.connectionState.value.readyContainer()?.deviceDataSource

fun ServiceConnectionManager.customDns() = this.connectionState.value.readyContainer()?.customDns

fun ServiceConnectionManager.splitTunneling() =
    this.connectionState.value.readyContainer()?.splitTunneling

fun ServiceConnectionManager.voucherRedeemer() =
    this.connectionState.value.readyContainer()?.voucherRedeemer

package net.mullvad.mullvadvpn.ui.serviceconnection

fun ServiceConnectionManager.appVersionInfoCache() =
    this.connectionState.value.readyContainer()?.appVersionInfoCache

fun ServiceConnectionManager.authTokenCache() =
    this.connectionState.value.readyContainer()?.authTokenCache

fun ServiceConnectionManager.connectionProxy() =
    this.connectionState.value.readyContainer()?.connectionProxy

fun ServiceConnectionManager.deviceDataSource() =
    this.connectionState.value.readyContainer()?.deviceDataSource

fun ServiceConnectionManager.customDns() = this.connectionState.value.readyContainer()?.customDns

fun ServiceConnectionManager.settingsListener() =
    this.connectionState.value.readyContainer()?.settingsListener

fun ServiceConnectionManager.splitTunneling() =
    this.connectionState.value.readyContainer()?.splitTunneling

fun ServiceConnectionManager.voucherRedeemer() =
    this.connectionState.value.readyContainer()?.voucherRedeemer

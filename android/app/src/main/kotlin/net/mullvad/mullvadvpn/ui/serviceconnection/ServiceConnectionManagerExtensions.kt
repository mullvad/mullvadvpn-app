package net.mullvad.mullvadvpn.ui.serviceconnection

fun ServiceConnectionManager.accountDataSource() =
    this.connectionState.value.readyContainer()?.accountDataSource

fun ServiceConnectionManager.appVersionInfoCache() =
    this.connectionState.value.readyContainer()?.appVersionInfoCache

fun ServiceConnectionManager.authTokenCache() =
    this.connectionState.value.readyContainer()?.authTokenCache

fun ServiceConnectionManager.connectionProxy() =
    this.connectionState.value.readyContainer()?.connectionProxy

fun ServiceConnectionManager.deviceDataSource() =
    this.connectionState.value.readyContainer()?.deviceDataSource

fun ServiceConnectionManager.customDns() =
    this.connectionState.value.readyContainer()?.customDns

fun ServiceConnectionManager.relayListListener() =
    this.connectionState.value.readyContainer()?.relayListListener

fun ServiceConnectionManager.settingsListener() =
    this.connectionState.value.readyContainer()?.settingsListener

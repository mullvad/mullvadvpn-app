package net.mullvad.mullvadvpn.lib.common.constant

// Do not use in cases where the application id is expected since the application id will differ
// between different builds.
private const val MULLVAD_PACKAGE_NAME = "net.mullvad.mullvadvpn"

// Classes
const val MAIN_ACTIVITY_CLASS = "$MULLVAD_PACKAGE_NAME.ui.MainActivity"
const val VPN_SERVICE_CLASS = "$MULLVAD_PACKAGE_NAME.service.MullvadVpnService"

// Actions
const val KEY_CONNECT_ACTION = "$MULLVAD_PACKAGE_NAME.connect_action"
const val KEY_DISCONNECT_ACTION = "$MULLVAD_PACKAGE_NAME.disconnect_action"
const val KEY_REQUEST_VPN_PERMISSION = "$MULLVAD_PACKAGE_NAME.request_vpn_permission"

package net.mullvad.mullvadvpn.lib.common.constant

// Do not use in cases where the application id is expected since the application id will differ
// between different builds.
internal const val MULLVAD_PACKAGE_NAME = "net.mullvad.mullvadvpn"

// Classes
const val MAIN_ACTIVITY_CLASS = "$MULLVAD_PACKAGE_NAME.ui.MainActivity"
const val VPN_SERVICE_CLASS = "$MULLVAD_PACKAGE_NAME.service.MullvadVpnService"
const val WIDGET_ACTION_RECEIVER = "$MULLVAD_PACKAGE_NAME.receiver.WidgetActionsReceiver"

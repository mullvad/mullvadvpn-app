package net.mullvad.mullvadvpn.lib.common.constant

// Do not use in cases where the application id is expected since the application id will differ
// between different builds.
internal const val MULLVAD_PACKAGE_NAME = "net.mullvad.mullvadvpn"

// Classes
const val MAIN_ACTIVITY_CLASS = "$MULLVAD_PACKAGE_NAME.ui.MainActivity"
const val VPN_SERVICE_CLASS = "$MULLVAD_PACKAGE_NAME.service.MullvadVpnService"

// Activity alt classes
const val MAIN_ACTIVITY_ALT_DEFAULT_CLASS =
    "$MULLVAD_PACKAGE_NAME.ui.obfuscation.MainActivityAltDefault"
const val MAIN_ACTIVITY_ALT_GAME_CLASS = "$MULLVAD_PACKAGE_NAME.ui.obfuscation.MainActivityAltGame"
const val MAIN_ACTIVITY_ALT_NINJA_CLASS =
    "$MULLVAD_PACKAGE_NAME.ui.obfuscation.MainActivityAltNinja"
const val MAIN_ACTIVITY_ALT_WEATHER_CLASS =
    "$MULLVAD_PACKAGE_NAME.ui.obfuscation.MainActivityAltWeather"
const val MAIN_ACTIVITY_ALT_NOTES_CLASS =
    "$MULLVAD_PACKAGE_NAME.ui.obfuscation.MainActivityAltNotes"
const val MAIN_ACTIVITY_ALT_BROWSER_CLASS =
    "$MULLVAD_PACKAGE_NAME.ui.obfuscation.MainActivityAltBrowser"

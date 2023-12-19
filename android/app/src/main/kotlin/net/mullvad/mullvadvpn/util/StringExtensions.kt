package net.mullvad.mullvadvpn.util

fun String.appendHideNavOnPlayBuild(isPlayBuild: Boolean): String =
    if (isPlayBuild) {
        "$this?hide_nav"
    } else {
        this
    }

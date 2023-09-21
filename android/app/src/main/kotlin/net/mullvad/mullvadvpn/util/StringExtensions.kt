package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD

fun String.appendHideNavOnReleaseBuild(): String =
    if (IS_PLAY_BUILD) {
        "$this?hide_nav"
    } else {
        this
    }

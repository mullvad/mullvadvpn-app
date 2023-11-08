package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD

private const val IGNORED_VOUCHER_CHARACTER_REGEX_PATTERN = """[- \n\r]"""

fun String.appendHideNavOnPlayBuild(): String =
    if (IS_PLAY_BUILD) {
        "$this?hide_nav"
    } else {
        this
    }

fun String.trimAsVoucher(): String =
    this.replace(Regex(IGNORED_VOUCHER_CHARACTER_REGEX_PATTERN), "")

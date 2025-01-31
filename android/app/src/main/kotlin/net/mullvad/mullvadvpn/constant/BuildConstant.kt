package net.mullvad.mullvadvpn.constant

import net.mullvad.mullvadvpn.BuildConfig

const val IS_PLAY_BUILD = BuildConfig.FLAVOR_billing == "play"
const val IS_FDROID_BUILD = BuildConfig.BUILD_TYPE == "fdroid"

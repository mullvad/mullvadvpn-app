package net.mullvad.mullvadvpn.constant

import kotlin.time.Duration.Companion.seconds

const val VERIFICATION_MAX_ATTEMPTS = 4
val VERIFICATION_INITIAL_BACK_OFF_DURATION = 3.seconds
const val VERIFICATION_BACK_OFF_FACTOR = 3.toDouble()

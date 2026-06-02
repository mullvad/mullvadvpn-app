package net.mullvad.mullvadvpn.lib.common.compose

import kotlinx.coroutines.flow.StateFlow

/**
 * TEMPORARY (speed-run competition): exposes whether a timed run is in progress so screens in other
 * modules (the login and privacy-disclaimer screens) can gate their input on it. Implemented by the
 * SpeedrunController in the app module and provided via Koin.
 */
interface SpeedrunGate {
    val isRunning: StateFlow<Boolean>
}

package net.mullvad.mullvadvpn.util

import kotlin.reflect.KClass
import net.mullvad.mullvadvpn.model.DeviceState

private const val ZERO_DEBOUNCE_DELAY_MILLISECONDS = 0L

fun <T> DeviceState.addDebounceForStates(delay: Long, vararg states: KClass<T>): Long where
T : DeviceState {
    val result = states.any { this::class == it }
    return if (result) {
        delay
    } else {
        ZERO_DEBOUNCE_DELAY_MILLISECONDS
    }
}

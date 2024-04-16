package net.mullvad.mullvadvpn.model

import arrow.optics.optics

@optics
sealed interface Constraint<out T> {
    data object Any : Constraint<Nothing>

    @optics
    data class Only<T>(val value: T) : Constraint<T> {
        companion object
    }

    companion object
}

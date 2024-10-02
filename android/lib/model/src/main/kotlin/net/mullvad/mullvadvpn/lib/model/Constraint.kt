package net.mullvad.mullvadvpn.lib.model

sealed interface Constraint<out T> {
    data object Any : Constraint<Nothing>

    data class Only<T>(val value: T) : Constraint<T> {
        companion object
    }

    fun getOrNull(): T? =
        when (this) {
            Any -> null
            is Only -> value
        }

    companion object
}

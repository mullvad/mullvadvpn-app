package net.mullvad.mullvadvpn.model

sealed class Constraint<T>() {
    class Any<T>() : Constraint<T>()

    data class Only<T>(val value: T) : Constraint<T>() {
        fun get0() = value
    }
}

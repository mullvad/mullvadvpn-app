package net.mullvad.mullvadvpn.model

sealed class Constraint<T>() {
    class Any<T>() : Constraint<T>()
    class Only<T>(val value: T) : Constraint<T>()
}

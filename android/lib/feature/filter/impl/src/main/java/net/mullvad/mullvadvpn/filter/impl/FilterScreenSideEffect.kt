package net.mullvad.mullvadvpn.filter.impl

sealed interface FilterScreenSideEffect {
    data object CloseScreen : FilterScreenSideEffect
}

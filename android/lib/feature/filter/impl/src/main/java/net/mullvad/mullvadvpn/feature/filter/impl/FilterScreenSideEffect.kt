package net.mullvad.mullvadvpn.feature.filter.impl

sealed interface FilterScreenSideEffect {
    data object CloseScreen : FilterScreenSideEffect
}

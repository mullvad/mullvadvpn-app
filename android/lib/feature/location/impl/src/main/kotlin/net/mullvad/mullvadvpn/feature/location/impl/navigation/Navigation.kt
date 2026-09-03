package net.mullvad.mullvadvpn.feature.location.impl.navigation

import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.anticensorship.api.AntiCensorshipNavKey
import net.mullvad.mullvadvpn.feature.daita.api.DaitaNavKey
import net.mullvad.mullvadvpn.lib.usecase.FilterChip

fun Navigator.navigateFromFilterChip(filterChip: FilterChip) {
    when (filterChip) {
        FilterChip.Daita -> navigate(DaitaNavKey())
        FilterChip.Lwo,
        FilterChip.Quic,
        is FilterChip.Shadowsocks -> navigate(AntiCensorshipNavKey())
        else -> {}
    }
}

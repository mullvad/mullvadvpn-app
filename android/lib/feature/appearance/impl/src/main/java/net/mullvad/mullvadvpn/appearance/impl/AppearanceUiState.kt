package net.mullvad.mullvadvpn.appearance.impl

import net.mullvad.mullvadvpn.appearance.impl.obfuscation.AppObfuscation

data class AppearanceUiState(
    val availableObfuscations: List<AppObfuscation> = emptyList(),
    val currentAppObfuscation: AppObfuscation? = null,
    val applyingChange: Boolean = false,
)

package net.mullvad.mullvadvpn.feature.appearance.impl

import net.mullvad.mullvadvpn.feature.appearance.impl.obfuscation.AppObfuscation

data class AppearanceUiState(
    val availableObfuscations: List<AppObfuscation> = emptyList(),
    val currentAppObfuscation: AppObfuscation? = null,
    val applyingChange: Boolean = false,
)

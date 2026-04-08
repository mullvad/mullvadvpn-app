package net.mullvad.mullvadvpn.feature.appicon.impl

import net.mullvad.mullvadvpn.feature.appicon.impl.obfuscation.AppObfuscation

data class AppIconUiState(
    val availableObfuscations: List<AppObfuscation> = emptyList(),
    val currentAppObfuscation: AppObfuscation? = null,
    val applyingChange: Boolean = false,
)

package net.mullvad.mullvadvpn.feature.vpnsettings.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator

@Parcelize
data class VpnSettingsNavKey(
    val scrollToFeature: FeatureIndicator? = null,
    val isModal: Boolean = false,
) : NavKey2

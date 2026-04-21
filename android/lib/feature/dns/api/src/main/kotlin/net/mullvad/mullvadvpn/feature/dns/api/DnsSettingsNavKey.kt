package net.mullvad.mullvadvpn.feature.dns.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator

@Parcelize
data class DnsSettingsNavKey(
    val selectedFeature: FeatureIndicator? = null,
    val isModal: Boolean = false,
) : NavKey2

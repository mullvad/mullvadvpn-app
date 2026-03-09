package net.mullvad.mullvadvpn.feature.vpnsettings.api

import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator

@Serializable data class VpnSettingsNavKey(val scrollToFeature: FeatureIndicator?, val isModal: Boolean) : NavKey

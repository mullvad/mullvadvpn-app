package net.mullvad.mullvadvpn.feature.anticensorship.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator

@Parcelize
data class AnticensorshipNavKey(
    val selectedFeature: FeatureIndicator? = null,
    val isModal: Boolean = false,
) : NavKey2

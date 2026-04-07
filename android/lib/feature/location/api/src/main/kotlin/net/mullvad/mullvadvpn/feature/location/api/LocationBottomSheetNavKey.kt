package net.mullvad.mullvadvpn.feature.location.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2

@Parcelize data class LocationBottomSheetNavKey(val state: LocationBottomSheetState) : NavKey2

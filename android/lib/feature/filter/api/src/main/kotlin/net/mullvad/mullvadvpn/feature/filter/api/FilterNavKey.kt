package net.mullvad.mullvadvpn.feature.filter.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.lib.model.FilterTarget

@Parcelize data class FilterNavKey(val filterTarget: FilterTarget) : NavKey2

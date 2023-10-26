package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.common.util.toGeographicLocationConstraint
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.RelayConstraints
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.WireguardConstraints
import net.mullvad.mullvadvpn.relaylist.RelayCountry
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.relaylist.findItemForLocation
import net.mullvad.mullvadvpn.relaylist.toRelayCountries
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener

class RelayListUseCase(
    private val relayListListener: RelayListListener,
    private val settingsRepository: SettingsRepository
) {

    fun updateSelectedRelayLocation(value: GeographicLocationConstraint) {
        relayListListener.updateSelectedRelayLocation(value)
    }

    fun updateSelectedWireguardConstraints(value: WireguardConstraints) {
        relayListListener.updateSelectedWireguardConstraints(value)
    }

    fun relayListWithSelection(): Flow<RelayList> =
        combine(relayListListener.relayListEvents, settingsRepository.settingsUpdates) {
            relayList,
            settings ->
            val ownership =
                settings?.relaySettings?.relayConstraints()?.ownership ?: Constraint.Any()
            val providers =
                settings?.relaySettings?.relayConstraints()?.providers ?: Constraint.Any()
            val relayCountries =
                relayList.toRelayCountries(ownership = ownership, providers = providers)
            val selectedItem =
                relayCountries.findSelectedRelayItem(
                    relaySettings = settings?.relaySettings,
                )
            RelayList(relayCountries, selectedItem)
        }

    fun selectedRelayItem(): Flow<RelayItem?> = relayListWithSelection().map { it.selectedItem }

    private fun List<RelayCountry>.findSelectedRelayItem(
        relaySettings: RelaySettings?,
    ): RelayItem? {
        val location = relaySettings?.relayConstraints()?.location
        return location?.let { this.findItemForLocation(location.toGeographicLocationConstraint()) }
    }

    private fun RelaySettings.relayConstraints(): RelayConstraints? =
        (this as? RelaySettings.Normal)?.relayConstraints
}

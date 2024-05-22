package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.WireguardConstraints
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.relaylist.RelayList
import net.mullvad.mullvadvpn.relaylist.filterOnOwnershipAndProvider
import net.mullvad.mullvadvpn.relaylist.findItemForGeographicLocationConstraint
import net.mullvad.mullvadvpn.relaylist.toRelayCountries
import net.mullvad.mullvadvpn.relaylist.toRelayItemLists
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener

class RelayListUseCase(
    private val relayListListener: RelayListListener,
    private val settingsRepository: SettingsRepository
) {

    suspend fun updateSelectedRelayLocation(value: LocationConstraint) {
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
            val relayCountries = relayList.toRelayCountries()
            val customLists =
                settings?.customLists?.customLists?.toRelayItemLists(relayCountries) ?: emptyList()
            val relayCountriesFiltered =
                relayCountries.mapNotNull { it.filterOnOwnershipAndProvider(ownership, providers) }
            val selectedItem =
                findSelectedRelayItem(
                    relaySettings = settings?.relaySettings,
                    relayCountries = relayCountries,
                    customLists = customLists,
                )
            RelayList(
                customLists = customLists,
                allCountries = relayCountries,
                filteredCountries = relayCountriesFiltered,
                selectedItem = selectedItem
            )
        }

    fun selectedRelayItem(): Flow<RelayItem?> = relayListWithSelection().map { it.selectedItem }

    fun fullRelayList(): Flow<List<RelayItem.Country>> =
        relayListWithSelection().map { it.allCountries }

    fun customLists(): Flow<List<RelayItem.CustomList>> =
        relayListWithSelection().map { it.customLists }

    fun fetchRelayList() {
        relayListListener.fetchRelayList()
    }

    private fun findSelectedRelayItem(
        relaySettings: RelaySettings?,
        relayCountries: List<RelayItem.Country>,
        customLists: List<RelayItem.CustomList>
    ): RelayItem? {
        val locationConstraint = relaySettings?.relayConstraints()?.location
        return if (locationConstraint is Constraint.Only) {
            when (val location = locationConstraint.value) {
                is LocationConstraint.CustomList -> {
                    customLists.firstOrNull { it.id == location.listId }
                }
                is LocationConstraint.Location -> {
                    relayCountries.findItemForGeographicLocationConstraint(location.location)
                }
            }
        } else {
            null
        }
    }
}

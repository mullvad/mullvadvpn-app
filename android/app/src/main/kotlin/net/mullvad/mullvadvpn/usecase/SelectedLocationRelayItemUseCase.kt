package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.relaylist.findByGeoLocationId
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase

class SelectedLocationRelayItemUseCase(
    private val customListRelayItemUseCase: CustomListsRelayItemUseCase,
    private val relayListRepository: RelayListRepository,
) {
    operator fun invoke() =
        combine(
            customListRelayItemUseCase(),
            relayListRepository.relayList,
            relayListRepository.selectedLocation,
        ) { customLists, relayList, selectedLocation ->
            if (selectedLocation is Constraint.Only) {
                when (val id = selectedLocation.value) {
                    is CustomListId -> customLists.firstOrNull { it.id == id }
                    is GeoLocationId -> relayList.findByGeoLocationId(id)
                }
            } else {
                null
            }
        }
}

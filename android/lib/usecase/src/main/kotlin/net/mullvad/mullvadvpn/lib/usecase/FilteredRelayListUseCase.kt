package net.mullvad.mullvadvpn.lib.usecase

import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import mullvad_daemon.management_interface.location
import net.mullvad.mullvadvpn.lib.common.util.relaylist.filter
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.EntryConstraints
import net.mullvad.mullvadvpn.lib.model.ExitConstraints
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.RelayPartitions
import net.mullvad.mullvadvpn.lib.model.RelaySelectorPredicate
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository

class FilteredRelayListUseCase(
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
    private val managementService: ManagementService,
) {
    operator fun invoke(relayListType: RelayListType) =
        when (relayListType) {
            is RelayListType.Multihop -> TODO()
            RelayListType.Single ->
                combine(
                    settingsRepository.settingsUpdates
                        .filterNotNull()
                        .map { RelaySelectorPredicate.SingleHop(it.toPredicate()) }
                        .distinctUntilChanged()
                        .map { managementService.partitionRelays(it) },
                    relayListRepository.relayList,
                ) { partitions, relayList ->
                    relayList.filter(partitions.relevantHostnames())
                }
        }

    private fun RelayPartitions.relevantHostnames() = matches

    private fun List<RelayItem.Location.Country>.filter(
        validHostnames: List<GeoLocationId.Hostname>
    ) = mapNotNull { it.filter(validHostnames) }
}

private fun Settings.toPredicate(): EntryConstraints =
    EntryConstraints(
        generalConstraints =
            ExitConstraints(
                location = Constraint.Any,
                providers = relaySettings.relayConstraints.providers,
                ownership = relaySettings.relayConstraints.ownership,
            ),
        obfuscation = Constraint.Only(obfuscationSettings),
        daitaSettings = Constraint.Only(tunnelOptions.daitaSettings),
        ipVersion = relaySettings.relayConstraints.wireguardConstraints.ipVersion,
    )

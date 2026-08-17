package net.mullvad.mullvadvpn.lib.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.common.util.isWhenNeededMultihop
import net.mullvad.mullvadvpn.lib.common.util.relaylist.FilteredCountry
import net.mullvad.mullvadvpn.lib.common.util.relaylist.RelayMetadataMap
import net.mullvad.mullvadvpn.lib.common.util.relaylist.filter
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DiscardedRelay
import net.mullvad.mullvadvpn.lib.model.EntryConstraints
import net.mullvad.mullvadvpn.lib.model.ExitConstraints
import net.mullvad.mullvadvpn.lib.model.MultihopConstraints
import net.mullvad.mullvadvpn.lib.model.NeedsOtherEntry
import net.mullvad.mullvadvpn.lib.model.PartitionHostname
import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.RelayPartitions
import net.mullvad.mullvadvpn.lib.model.RelaySelectorPredicate
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository

data class FilteredCountries(
    val countries: List<RelayItem.Location.Country> = emptyList(),
    val relayMetadata: RelayMetadataMap = emptyMap(),
)

class FilteredRelayListUseCase(
    private val relayListRepository: RelayListRepository,
    private val settingsRepository: SettingsRepository,
    private val managementService: ManagementService,
) {
    operator fun invoke(relayListType: RelayListType): Flow<FilteredCountries> =
        combine(
            settingsRepository.settingsUpdates
                .filterNotNull()
                .map {
                    when (relayListType) {
                        is RelayListType.Multihop ->
                            when (relayListType.hopType) {
                                RelayHopType.ENTRY ->
                                    RelaySelectorPredicate.Entry(
                                        multihopConstraints =
                                            MultihopConstraints(
                                                entryConstraints =
                                                    it.toEntryConstraint(Constraint.Any),
                                                exitConstraints = it.toExitConstraint(),
                                            )
                                    )
                                RelayHopType.EXIT ->
                                    if (it.isWhenNeededMultihop()) {
                                        RelaySelectorPredicate.Autohop(
                                            it.toEntryConstraint(Constraint.Any)
                                        )
                                    } else {
                                        RelaySelectorPredicate.Exit(
                                            multihopConstraints =
                                                MultihopConstraints(
                                                    entryConstraints = it.toEntryConstraint(),
                                                    exitConstraints =
                                                        it.toExitConstraint(Constraint.Any),
                                                )
                                        )
                                    }
                            }
                        RelayListType.Single ->
                            if (it.isWhenNeededMultihop()) {
                                RelaySelectorPredicate.Autohop(it.toEntryConstraint(Constraint.Any))
                            } else {
                                RelaySelectorPredicate.SingleHop(
                                    it.toEntryConstraint(Constraint.Any)
                                )
                            }
                    }
                }
                .distinctUntilChanged()
                .map {
                    // We expect this to always work
                    managementService.partitionRelays(it).getOrNull()!!
                },
            relayListRepository.relayList,
        ) { partitions, relayList ->
            val filtered = relayList.filter(partitions.relevantHostnames())
            val countries = filtered.map { it.country }
            val metadata = buildMap {
                filtered.map { it.relayMetadata }.forEach(::putAll)
            }
            FilteredCountries(countries = countries, relayMetadata = metadata)
        }

    private fun RelayPartitions.relevantHostnames(): Map<PartitionHostname, NeedsOtherEntry> {
        val discardsToShow =
            discards.filter { it.shouldBeShown() }.associate { it.hostname to false }
        return matches + discardsToShow
    }

    private fun DiscardedRelay.shouldBeShown(): Boolean =
        with(why) {
            (conflictWithOtherHop or inactive) &&
                !location &&
                !providers &&
                !ownership &&
                !ipVersion &&
                !daita &&
                !obfuscation &&
                !port
        }

    private fun List<RelayItem.Location.Country>.filter(
        validHostnames: Map<PartitionHostname, NeedsOtherEntry>
    ): List<FilteredCountry> = mapNotNull {
        it.filter(validHostnames)
    }
}

private fun Settings.toEntryConstraint(
    overrideExitLocation: Constraint<RelayItemId>? = null
): EntryConstraints =
    EntryConstraints(
        generalConstraints =
            ExitConstraints(
                location = overrideExitLocation ?: relaySettings.relayConstraints.location,
                providers = relaySettings.relayConstraints.wireguardConstraints.entryProviders,
                ownership = relaySettings.relayConstraints.wireguardConstraints.entryOwnership,
            ),
        obfuscation = Constraint.Only(obfuscationSettings),
        daitaSettings = Constraint.Only(tunnelOptions.daitaSettings),
        ipVersion = relaySettings.relayConstraints.wireguardConstraints.ipVersion,
    )

private fun Settings.toExitConstraint(
    overrideEntryLocation: Constraint<RelayItemId>? = null
): ExitConstraints =
    ExitConstraints(
        location = overrideEntryLocation ?: relaySettings.relayConstraints.location,
        providers = relaySettings.relayConstraints.providers,
        ownership = relaySettings.relayConstraints.ownership,
    )

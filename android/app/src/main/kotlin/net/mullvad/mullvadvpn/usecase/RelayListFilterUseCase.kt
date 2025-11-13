package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository

class RelayListFilterUseCase(private val relayListFilterRepository: RelayListFilterRepository) {
    operator fun invoke(relayListType: RelayListType) =
        when (relayListType) {
            is RelayListType.Multihop if
                relayListType.multihopRelayListType == MultihopRelayListType.ENTRY
             -> entryFilters()
            else -> exitFilters()
        }

    private fun entryFilters() =
        combine(
            relayListFilterRepository.selectedMultihopEntryOwnership,
            relayListFilterRepository.selectedMultihopEntryProviders,
        ) { ownership, providers ->
            ownership to providers
        }

    private fun exitFilters() =
        combine(
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders,
        ) { ownership, providers ->
            ownership to providers
        }
}

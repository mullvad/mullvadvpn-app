package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.util.shouldFilterByDaita

typealias ModelOwnership = Ownership

class FilterChipUseCase(
    private val relayListFilterRepository: RelayListFilterRepository,
    private val providerOwnershipUseCase: ProviderOwnershipUseCase,
    private val settingsRepository: SettingsRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) {
    operator fun invoke(relayListType: RelayListType): Flow<List<FilterChip>> =
        combine(
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders,
            providerOwnershipUseCase(),
            settingsRepository.settingsUpdates,
            wireguardConstraintsRepository.wireguardConstraints,
        ) {
            selectedOwnership,
            selectedConstraintProviders,
            providerOwnership,
            settings,
            wireguardConstraints ->
            filterChips(
                selectedOwnership = selectedOwnership,
                selectedConstraintProviders = selectedConstraintProviders,
                providerOwnershipRelationship = providerOwnership,
                daitaDirectOnly = settings?.daitaAndDirectOnly() == true,
                isMultihopEnabled = wireguardConstraints?.isMultihopEnabled == true,
                relayListType = relayListType,
            )
        }

    private fun filterChips(
        selectedOwnership: Constraint<Ownership>,
        selectedConstraintProviders: Constraint<Providers>,
        providerOwnershipRelationship: Map<ProviderId, Set<Ownership>>,
        daitaDirectOnly: Boolean,
        isMultihopEnabled: Boolean,
        relayListType: RelayListType,
    ): List<FilterChip> {
        val ownershipFilter = selectedOwnership.getOrNull()
        val providerCountFilter =
            when (selectedConstraintProviders) {
                is Constraint.Any -> null
                is Constraint.Only ->
                    selectedConstraintProviders.value.providers
                        .filter { providerId ->
                            if (ownershipFilter == null) {
                                true
                            } else {
                                providerOwnershipRelationship[providerId]!!.contains(
                                    ownershipFilter
                                )
                            }
                        }
                        .size
            }
        return buildList {
            if (ownershipFilter != null) {
                add(FilterChip.Ownership(ownershipFilter))
            }
            if (providerCountFilter != null) {
                add(FilterChip.Provider(providerCountFilter))
            }
            if (
                shouldFilterByDaita(
                    daitaDirectOnly = daitaDirectOnly,
                    relayListType = relayListType,
                    isMultihopEnabled = isMultihopEnabled,
                )
            ) {
                add(FilterChip.Daita)
            }
        }
    }

    private fun Settings.daitaAndDirectOnly() =
        tunnelOptions.wireguard.daitaSettings.enabled &&
            tunnelOptions.wireguard.daitaSettings.directOnly
}

sealed interface FilterChip {
    data class Ownership(val ownership: ModelOwnership) : FilterChip

    data class Provider(val count: Int) : FilterChip

    data object Daita : FilterChip

    data object Entry : FilterChip

    data object Exit : FilterChip
}

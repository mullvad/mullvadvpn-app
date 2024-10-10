package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.compose.state.toSelectedProviders
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository

typealias ModelOwnership = Ownership

class FilterChipUseCase(
    private val relayListFilterRepository: RelayListFilterRepository,
    private val availableProvidersUseCase: AvailableProvidersUseCase,
    private val settingsRepository: SettingsRepository,
) {
    operator fun invoke(): Flow<List<FilterChip>> =
        combine(
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders,
            availableProvidersUseCase(),
            settingsRepository.settingsUpdates,
        ) { selectedOwnership, selectedConstraintProviders, allProviders, settings ->
            filterChips(
                selectedOwnership = selectedOwnership,
                selectedConstraintProviders = selectedConstraintProviders,
                allProviders = allProviders,
                isDaitaEnabled = settings?.isDaitaEnabled() ?: false,
            )
        }

    private fun filterChips(
        selectedOwnership: Constraint<Ownership>,
        selectedConstraintProviders: Constraint<Providers>,
        allProviders: List<Provider>,
        isDaitaEnabled: Boolean,
    ): List<FilterChip> {
        val ownershipFilter = selectedOwnership.getOrNull()
        val providerCountFilter =
            when (selectedConstraintProviders) {
                is Constraint.Any -> null
                is Constraint.Only ->
                    filterSelectedProvidersByOwnership(
                            selectedConstraintProviders.toSelectedProviders(allProviders),
                            ownershipFilter,
                        )
                        .size
            }
        return buildList {
            if (ownershipFilter != null) {
                add(FilterChip.Ownership(ownershipFilter))
            }
            if (providerCountFilter != null) {
                add(FilterChip.Provider(providerCountFilter))
            }
            if (isDaitaEnabled) {
                add(FilterChip.Daita)
            }
        }
    }

    private fun filterSelectedProvidersByOwnership(
        selectedProviders: List<Provider>,
        selectedOwnership: Ownership?,
    ): List<Provider> =
        if (selectedOwnership == null) selectedProviders
        else selectedProviders.filter { it.ownership == selectedOwnership }
}

sealed interface FilterChip {
    data class Ownership(val ownership: ModelOwnership) : FilterChip

    data class Provider(val count: Int) : FilterChip

    data object Daita : FilterChip

    data object Entry : FilterChip

    data object Exit : FilterChip
}

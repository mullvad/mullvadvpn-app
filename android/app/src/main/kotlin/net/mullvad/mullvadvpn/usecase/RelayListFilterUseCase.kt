package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.RelayListCity
import net.mullvad.mullvadvpn.model.RelayListCountry
import net.mullvad.mullvadvpn.relaylist.Provider
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener

class RelayListFilterUseCase(
    private val relayListListener: RelayListListener,
    private val settingsRepository: SettingsRepository
) {
    fun updateOwnershipFilter(ownership: Constraint<Ownership>) {
        relayListListener.updateSelectedOwnershipFilter(ownership)
    }

    fun updateProviderFilter(providers: Constraint<Providers>) {
        relayListListener.updateSelectedProvidersFilter(providers)
    }

    fun updateOwnershipAndProviderFilter(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>
    ) {
        relayListListener.updateSelectedOwnershipAndProviderFilter(ownership, providers)
    }

    fun selectedOwnership(): Flow<Constraint<Ownership>> =
        settingsRepository.settingsUpdates.map { settings ->
            settings?.relaySettings?.relayConstraints()?.ownership ?: Constraint.Any()
        }

    fun selectedProviders(): Flow<Constraint<Providers>> =
        settingsRepository.settingsUpdates.map { settings ->
            settings?.relaySettings?.relayConstraints()?.providers ?: Constraint.Any()
        }

    fun availableProviders(): Flow<List<Provider>> =
        relayListListener.relayListEvents.map { relayList ->
            relayList.countries
                .flatMap(RelayListCountry::cities)
                .flatMap(RelayListCity::relays)
                .filter { relay -> relay.isWireguardRelay }
                .map { relay -> Provider(relay.provider, relay.owned) }
                .distinct()
        }
}

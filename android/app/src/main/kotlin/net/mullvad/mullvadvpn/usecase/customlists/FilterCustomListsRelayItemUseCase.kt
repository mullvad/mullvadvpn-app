package net.mullvad.mullvadvpn.usecase.customlists

import kotlin.collections.mapNotNull
import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.relaylist.filter
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.util.shouldFilterByDaita
import net.mullvad.mullvadvpn.util.shouldFilterByQuic

class FilterCustomListsRelayItemUseCase(
    private val customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    private val relayListFilterRepository: RelayListFilterRepository,
    private val settingsRepository: SettingsRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) {

    operator fun invoke(relayListType: RelayListType) =
        combine(
            customListsRelayItemUseCase(),
            relayListFilterRepository.selectedOwnership,
            relayListFilterRepository.selectedProviders,
            settingsRepository.settingsUpdates,
            wireguardConstraintsRepository.wireguardConstraints,
        ) { customLists, selectedOwnership, selectedProviders, settings, wireguardConstraints ->
            customLists.filter(
                ownership = selectedOwnership,
                providers = selectedProviders,
                daita =
                    shouldFilterByDaita(
                        daitaDirectOnly = settings?.daitaAndDirectOnly() == true,
                        relayListType = relayListType,
                    ),
                quic =
                    shouldFilterByQuic(
                        settings?.isQuicEnabled() == true,
                        relayListType = relayListType,
                    ),
                ipVersion = settings?.ipVersionConstraint() ?: Constraint.Any,
            )
        }

    private fun List<RelayItem.CustomList>.filter(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>,
        daita: Boolean,
        quic: Boolean,
        ipVersion: Constraint<IpVersion>,
    ) = mapNotNull {
        it.filter(ownership, providers, daita = daita, quic = quic, ipVersion = ipVersion)
    }

    private fun Settings.daitaAndDirectOnly() =
        tunnelOptions.wireguard.daitaSettings.enabled &&
            tunnelOptions.wireguard.daitaSettings.directOnly

    private fun Settings.isQuicEnabled() =
        obfuscationSettings.selectedObfuscationMode == ObfuscationMode.Quic

    private fun Settings.ipVersionConstraint() =
        relaySettings.relayConstraints.wireguardConstraints.ipVersion
}

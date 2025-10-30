package net.mullvad.mullvadvpn.usecase

import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.relaylist.filter
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.util.ipVersionConstraint
import net.mullvad.mullvadvpn.util.isDaitaAndDirectOnly
import net.mullvad.mullvadvpn.util.isLwoEnabled
import net.mullvad.mullvadvpn.util.isQuicEnabled
import net.mullvad.mullvadvpn.util.shouldFilterByDaita
import net.mullvad.mullvadvpn.util.shouldFilterByLwo
import net.mullvad.mullvadvpn.util.shouldFilterByQuic

class FilteredRelayListUseCase(
    private val relayListRepository: RelayListRepository,
    private val relayListFilterUseCase: RelayListFilterUseCase,
    private val settingsRepository: SettingsRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) {
    operator fun invoke(relayListType: RelayListType) =
        combine(
            relayListRepository.relayList,
            relayListFilterUseCase(relayListType),
            settingsRepository.settingsUpdates,
            wireguardConstraintsRepository.wireguardConstraints,
        ) { relayList, (selectedOwnership, selectedProviders), settings, wireguardConstraints ->
            relayList.filter(
                ownership = selectedOwnership,
                providers = selectedProviders,
                shouldFilterByDaita =
                    shouldFilterByDaita(
                        daitaDirectOnly = settings?.isDaitaAndDirectOnly() == true,
                        relayListType = relayListType,
                    ),
                shouldFilterByQuic =
                    shouldFilterByQuic(
                        isQuicEnabled = settings?.isQuicEnabled() == true,
                        relayListType = relayListType,
                    ),
                shouldFilterByLwo =
                    shouldFilterByLwo(
                        isLwoEnable = settings?.isLwoEnabled() == true,
                        relayListType = relayListType,
                    ),
                constraintIpVersion = settings?.ipVersionConstraint() ?: Constraint.Any,
            )
        }

    private fun List<RelayItem.Location.Country>.filter(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>,
        shouldFilterByDaita: Boolean,
        shouldFilterByQuic: Boolean,
        shouldFilterByLwo: Boolean,
        constraintIpVersion: Constraint<IpVersion>,
    ) = mapNotNull {
        it.filter(
            ownership,
            providers,
            shouldFilterByDaita,
            shouldFilterByQuic,
            shouldFilterByLwo,
            constraintIpVersion,
        )
    }
}

package net.mullvad.mullvadvpn.usecase.customlists

import kotlin.collections.mapNotNull
import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.relaylist.filter
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.RelayListFilterUseCase
import net.mullvad.mullvadvpn.util.ipVersionConstraint
import net.mullvad.mullvadvpn.util.isDaitaAndDirectOnly
import net.mullvad.mullvadvpn.util.isLwoEnabled
import net.mullvad.mullvadvpn.util.isQuicEnabled
import net.mullvad.mullvadvpn.util.shouldFilterByDaita
import net.mullvad.mullvadvpn.util.shouldFilterByLwo
import net.mullvad.mullvadvpn.util.shouldFilterByQuic

class FilterCustomListsRelayItemUseCase(
    private val customListsRelayItemUseCase: CustomListsRelayItemUseCase,
    private val relayListFilterUseCase: RelayListFilterUseCase,
    private val settingsRepository: SettingsRepository,
) {

    operator fun invoke(relayListType: RelayListType) =
        combine(
            customListsRelayItemUseCase(),
            relayListFilterUseCase(relayListType),
            settingsRepository.settingsUpdates,
        ) { customLists, (selectedOwnership, selectedProviders), settings ->
            customLists.filter(
                ownership = selectedOwnership,
                providers = selectedProviders,
                daita =
                    shouldFilterByDaita(
                        daitaDirectOnly = settings?.isDaitaAndDirectOnly() == true,
                        relayListType = relayListType,
                    ),
                quic =
                    shouldFilterByQuic(
                        isQuicEnabled = settings?.isQuicEnabled() == true,
                        relayListType = relayListType,
                    ),
                lwo =
                    shouldFilterByLwo(
                        isLwoEnable = settings?.isLwoEnabled() == true,
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
        lwo: Boolean,
        ipVersion: Constraint<IpVersion>,
    ) = mapNotNull {
        it.filter(
            ownership,
            providers,
            daita = daita,
            quic = quic,
            lwo = lwo,
            ipVersion = ipVersion,
        )
    }
}

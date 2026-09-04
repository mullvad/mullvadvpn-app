package net.mullvad.mullvadvpn.lib.usecase

import kotlinx.coroutines.flow.Flow
import net.mullvad.mullvadvpn.lib.common.util.combine
import net.mullvad.mullvadvpn.lib.common.util.isDaitaEnabled
import net.mullvad.mullvadvpn.lib.common.util.isLwoEnabled
import net.mullvad.mullvadvpn.lib.common.util.isQuicEnabled
import net.mullvad.mullvadvpn.lib.common.util.isShadowsocksEnabled
import net.mullvad.mullvadvpn.lib.common.util.isWhenNeededMultihop
import net.mullvad.mullvadvpn.lib.common.util.shadowSocksPort
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayHopType
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.hopType
import net.mullvad.mullvadvpn.lib.model.isMultihopEntry
import net.mullvad.mullvadvpn.lib.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository

typealias ModelOwnership = Ownership

class FilterChipUseCase(
    private val relayListRepository: RelayListRepository,
    private val relayListFilterRepository: RelayListFilterRepository,
    private val providerToOwnershipsUseCase: ProviderToOwnershipsUseCase,
    private val settingsRepository: SettingsRepository,
    private val multihopInEffectUseCase: MultihopInEffectUseCase,
) {
    operator fun invoke(relayListType: RelayListType): Flow<List<FilterChip>> =
        combine(
            relayListFilterRepository.selectedOwnership(relayListType.hopType()),
            relayListFilterRepository.selectedProviders(relayListType.hopType()),
            providerToOwnershipsUseCase(),
            settingsRepository.settingsUpdates,
            multihopInEffectUseCase(),
            relayListRepository.shadowsocksPortRanges,
        ) {
            selectedOwnership,
            selectedProviders,
            providerOwnership,
            settings,
            multihopInEffect,
            shadowsocksPortRanges ->
            filterChips(
                selectedOwnership = selectedOwnership,
                selectedConstraintProviders = selectedProviders,
                providerToOwnerships = providerOwnership,
                settings = settings,
                relayListType = relayListType,
                multihopInEffect = multihopInEffect,
                shadowsocksPortRanges = shadowsocksPortRanges,
            )
        }

    private fun filterChips(
        selectedOwnership: Constraint<Ownership>,
        selectedConstraintProviders: Constraint<Providers>,
        providerToOwnerships: Map<ProviderId, Set<Ownership>>,
        settings: Settings?,
        relayListType: RelayListType,
        multihopInEffect: MultihopInEffectStatus,
        shadowsocksPortRanges: List<PortRange>,
    ): List<FilterChip> {

        // Do not show any entry filters for when needed multihop.
        if (
            relayListType.isMultihopEntry &&
                multihopInEffect == MultihopInEffectStatus.WhenNeededInEffect
        ) {
            return emptyList()
        }

        val ownershipFilter = selectedOwnership.getOrNull()
        val providerCountFilter =
            when (selectedConstraintProviders) {
                is Constraint.Any -> null
                is Constraint.Only ->
                    selectedConstraintProviders.value
                        .filter { providerId ->
                            if (ownershipFilter == null) {
                                true
                            } else {
                                val providerOwnerships = providerToOwnerships[providerId]
                                // If the provider has been removed from the relay list we add it
                                // so it is visible for the user, because we won't know what
                                // ownerships it had.
                                providerOwnerships?.contains(ownershipFilter) ?: true
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
                shouldShowFilterByFeature(
                    isFeatureEnabled = settings?.isDaitaEnabled() == true,
                    isWhenNeededMultihopEnabled = settings?.isWhenNeededMultihop() == true,
                    relayListType = relayListType,
                )
            ) {
                add(FilterChip.Daita)
            }
            if (
                shouldShowFilterByFeature(
                    isFeatureEnabled = settings?.isQuicEnabled() == true,
                    isWhenNeededMultihopEnabled = settings?.isWhenNeededMultihop() == true,
                    relayListType = relayListType,
                )
            ) {
                add(FilterChip.Quic)
            }
            if (
                shouldShowFilterByFeature(
                    isFeatureEnabled = settings?.isLwoEnabled() == true,
                    isWhenNeededMultihopEnabled = settings?.isWhenNeededMultihop() == true,
                    relayListType = relayListType,
                )
            ) {
                add(FilterChip.Lwo)
            }
            val shadowsocksPort = settings?.shadowSocksPort()?.getOrNull()
            if (
                shadowsocksPort != null &&
                    // Do not show the shadowsocks filter chip if a standard port is used because
                    // this is supported by all servers.
                    shadowsocksPortRanges.none { it.contains(shadowsocksPort) } &&
                    shouldShowFilterByFeature(
                        isFeatureEnabled = settings.isShadowsocksEnabled(),
                        isWhenNeededMultihopEnabled = settings.isWhenNeededMultihop(),
                        relayListType = relayListType,
                    )
            ) {
                add(FilterChip.Shadowsocks(shadowsocksPort))
            }
        }
    }

    private fun shouldShowFilterByFeature(
        isFeatureEnabled: Boolean,
        isWhenNeededMultihopEnabled: Boolean,
        relayListType: RelayListType,
    ) =
        when (relayListType) {
            RelayListType.Single -> isFeatureEnabled && !isWhenNeededMultihopEnabled
            is RelayListType.Multihop ->
                isFeatureEnabled && relayListType.hopType == RelayHopType.ENTRY
        }
}

sealed interface FilterChip {

    enum class Type {
        Relay,
        Setting,
    }

    val type: Type

    data class Ownership(val ownership: ModelOwnership) : FilterChip {
        override val type: Type
            get() = Type.Relay
    }

    data class Provider(val count: Int) : FilterChip {
        override val type: Type
            get() = Type.Relay
    }

    data class Shadowsocks(val port: Port) : FilterChip {
        override val type: Type
            get() = Type.Setting
    }

    data object Daita : FilterChip {
        override val type: Type
            get() = Type.Setting
    }

    data object Entry : FilterChip {
        override val type: Type
            get() = Type.Setting
    }

    data object Exit : FilterChip {
        override val type: Type
            get() = Type.Setting
    }

    data object Quic : FilterChip {
        override val type: Type
            get() = Type.Setting
    }

    data object Lwo : FilterChip {
        override val type: Type
            get() = Type.Setting
    }
}

package net.mullvad.mullvadvpn.lib.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository

class MultihopInEffectUseCase(
    private val connectionProxy: ConnectionProxy,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) {
    operator fun invoke(): Flow<MultihopInEffectStatus> =
        combine(
            connectionProxy.tunnelState,
            wireguardConstraintsRepository.wireguardConstraints.filterNotNull(),
        ) { tunnelState, wireguardConstrains ->
            checkMultihopActive(tunnelState, wireguardConstrains)
        }

    private fun checkMultihopActive(
        tunnelState: TunnelState,
        wireguardConstrains: WireguardConstraints,
    ): MultihopInEffectStatus =
        when (wireguardConstrains.multihop) {
            MultihopMode.ALWAYS -> MultihopInEffectStatus.AlwaysOnInEffect

            MultihopMode.WHEN_NEEDED if tunnelState.isWhenNeededMultihopInEffect() ->
                MultihopInEffectStatus.WhenNeededInEffect

            else -> MultihopInEffectStatus.Inactive
        }
}

enum class MultihopInEffectStatus {
    AlwaysOnInEffect,
    WhenNeededInEffect,
    Inactive;

    val isInEffect: Boolean
        get() = this != Inactive
}

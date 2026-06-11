package net.mullvad.mullvadvpn.lib.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.MultihopMode
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository

class MultihopActiveUseCase(
    private val connectionProxy: ConnectionProxy,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
) {
    operator fun invoke(): Flow<MultihopActiveStatus> =
        combine(
            connectionProxy.tunnelState,
            wireguardConstraintsRepository.wireguardConstraints.filterNotNull(),
        ) { tunnelState, wireguardConstrains ->
            checkMultihopActive(tunnelState, wireguardConstrains)
        }

    private fun checkMultihopActive(
        tunnelState: TunnelState,
        wireguardConstrains: WireguardConstraints,
    ): MultihopActiveStatus =
        when (wireguardConstrains.multihop) {
            MultihopMode.ALWAYS -> MultihopActiveStatus.AlwaysOnActive

            MultihopMode.WHEN_NEEDED if
                tunnelState.featureIndicators()?.contains(FeatureIndicator.MULTIHOP_AUTO) == true
             -> MultihopActiveStatus.WhenNeededActive

            else -> MultihopActiveStatus.Inactive
        }
}

enum class MultihopActiveStatus {
    AlwaysOnActive,
    WhenNeededActive,
    Inactive;

    val isActive: Boolean
        get() = this != Inactive
}

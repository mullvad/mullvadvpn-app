package net.mullvad.mullvadvpn.lib.feature.customvpn.impl.list

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.TunnelStats
import net.mullvad.mullvadvpn.lib.repository.CustomTunnelsRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository

class CustomTunnelsListViewModel(
    settingsRepository: SettingsRepository,
    private val customTunnelsRepository: CustomTunnelsRepository,
) : ViewModel() {
    private val _uiSideEffect = Channel<CustomTunnelsListSideEffect>()
    val uiSideEffect = merge(_uiSideEffect.receiveAsFlow())

    val uiState: StateFlow<Lc<Unit, CustomTunnelsListUiState>> =
        combine(
                settingsRepository.settingsUpdates.filterNotNull(),
                personalVpnTunnelStats(),
            ) { settings, stats ->
                Lc.Content(
                    CustomTunnelsListUiState(
                        customVpnConfigState =
                            settings.personalVpnConfig?.let {
                                if (settings.personalVpnEnabled) {
                                    CustomTunnelConfigState.Enabled(it)
                                } else {
                                    CustomTunnelConfigState.Disabled(it)
                                }
                            },
                        tunnelStats = stats,
                    )
                )
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                initialValue = Lc.Loading(Unit),
            )

    fun onEnableCustomVpn(enable: Boolean) {
        viewModelScope.launch { customTunnelsRepository.toggleCustomVpn(enable) }
    }

    fun importConfig(config: String) {
        viewModelScope.launch {
            customTunnelsRepository
                .importConfig(config)
                .fold(
                    { _uiSideEffect.send(CustomTunnelsListSideEffect.FailedToSave(it.toString())) },
                    { _uiSideEffect.send(CustomTunnelsListSideEffect.ConfigurationSaved) },
                )
        }
    }

    private fun personalVpnTunnelStats(): Flow<TunnelStats> = flow {
        emit(TunnelStats(0, 0))
        while (currentCoroutineContext().isActive) {
            customTunnelsRepository
                .getPersonalVpnStats()
                .fold(
                    { error ->
                        // We will just keep the old value if we fail to get the stats.
                        Logger.e { "Error getting personal VPN stats: $error" }
                    },
                    { stats -> emit(stats) },
                )

            delay(1.seconds)
        }
    }
}

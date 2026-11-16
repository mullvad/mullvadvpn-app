package net.mullvad.mullvadvpn.feature.customvpn.impl.list

import android.content.ContentResolver
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.TunnelStats
import net.mullvad.mullvadvpn.lib.repository.CustomVpnRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository

@OptIn(ExperimentalCoroutinesApi::class)
class CustomVpnListViewModel(
    isModal: Boolean,
    settingsRepository: SettingsRepository,
    private val customVpnRepository: CustomVpnRepository,
    private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {
    private val _uiSideEffect = Channel<CustomVpnListSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val settingsUpdates = settingsRepository.settingsUpdates.filterNotNull()

    val uiState: StateFlow<Lc<Unit, CustomVpnListUiState>> =
        combine(
                settingsUpdates,
                customVpnStats(),
            ) { settings, stats ->
                Lc.Content(
                    CustomVpnListUiState(
                        customVpnConfig = settings.customVpnConfig,
                        tunnelStats = stats,
                        isModal = isModal,
                    ),
                )
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                initialValue = Lc.Loading(Unit),
            )

    fun onToggleCustomTunnel(config: CustomVpnConfig) {
        viewModelScope.launch {
            customVpnRepository.setConfig(config.copy(enabled = !config.enabled))
        }
    }

    fun importConfig(name: String, contentResolver: ContentResolver, uri: Uri) {
        viewModelScope.launch(ioDispatcher) {
            contentResolver.openInputStream(uri)?.use { inputStream ->
                val body = inputStream.reader(Charsets.UTF_8).readText()

                val importResult =
                    customVpnRepository
                        .importConfig(name = name, config = body)
                        .fold(
                            { CustomVpnListSideEffect.FailedToSave(it.toString()) },
                            { CustomVpnListSideEffect.ConfigurationSaved },
                        )
                _uiSideEffect.send(importResult)
            }
        }
    }

    private fun customVpnStats(): Flow<TunnelStats?> =
        settingsUpdates
            .map { it.customVpnConfig?.enabled == true }
            .distinctUntilChanged()
            .flatMapLatest { customVpnActive ->
                if (customVpnActive) {
                    flow {
                        emit(TunnelStats(0, 0))
                        while (currentCoroutineContext().isActive) {
                            customVpnRepository
                                .getCustomVpnStats()
                                .fold(
                                    { error ->
                                        // We will just keep the old value if we fail to get the
                                        // stats.
                                        Logger.e { "Error getting custom VPN stats: $error" }
                                    },
                                    { stats -> emit(stats) },
                                )

                            delay(1.seconds)
                        }
                    }
                } else {
                    flowOf(null)
                }
            }
}

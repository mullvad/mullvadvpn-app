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
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.IsModal
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.CustomVpnConfig
import net.mullvad.mullvadvpn.lib.model.TunnelStats
import net.mullvad.mullvadvpn.lib.repository.CustomVpnRepository

class CustomVpnListViewModel(
    isModal: Boolean,
    private val customVpnRepository: CustomVpnRepository,
    private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {
    private val _uiSideEffect = Channel<CustomVpnListSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<Lc<IsModal, CustomVpnListUiState>> =
        combine(
                customVpnRepository.configFlow,
                customVpnStats(),
            ) { config, stats ->
                Lc.Content(
                    CustomVpnListUiState(
                        customVpnConfig = config,
                        tunnelStats = stats,
                        isModal = isModal,
                    ),
                )
            }
            .onStart { customVpnRepository.fetchConfig() }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                initialValue = Lc.Loading(IsModal(isModal)),
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

    @OptIn(ExperimentalCoroutinesApi::class)
    private fun customVpnStats(): Flow<TunnelStats?> =
        customVpnRepository.configFlow
            .map { it?.enabled == true }
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

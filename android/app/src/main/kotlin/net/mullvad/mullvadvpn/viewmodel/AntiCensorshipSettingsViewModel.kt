package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import com.ramcosta.composedestinations.generated.destinations.AntiCensorshipSettingsDestination
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.AntiCensorshipSettingsUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.selectedObfuscationMode
import net.mullvad.mullvadvpn.util.toLc
import net.mullvad.mullvadvpn.util.wireguardPort

sealed interface AntiCensorshipSideEffect {
    sealed interface ShowToast : AntiCensorshipSideEffect {
        data object GenericError : ShowToast
    }
}

class AntiCensorshipSettingsViewModel(
    private val settingsRepository: SettingsRepository,
    savedStateHandle: SavedStateHandle,
    private val ioDispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {
    private val navArgs = AntiCensorshipSettingsDestination.argsFrom(savedStateHandle)

    private val _uiSideEffect = Channel<AntiCensorshipSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<Lc<Unit, AntiCensorshipSettingsUiState>> =
        settingsRepository.settingsUpdates
            .filterNotNull()
            .map { settings ->
                AntiCensorshipSettingsUiState.from(
                        isModal = navArgs.isModal,
                        obfuscationMode = settings.selectedObfuscationMode(),
                        selectedUdp2TcpObfuscationPort = settings.obfuscationSettings.udp2tcp.port,
                        selectedShadowsocksObfuscationPort =
                            settings.obfuscationSettings.shadowsocks.port,
                        selectedWireguardPort = settings.wireguardPort(),
                    )
                    .toLc<Unit, AntiCensorshipSettingsUiState>()
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    fun onSelectObfuscationMode(obfuscationMode: ObfuscationMode) {
        Logger.e("Select obfuscation mode $obfuscationMode")
        viewModelScope.launch(ioDispatcher) {
            settingsRepository.setObfuscation(obfuscationMode).onLeft {
                _uiSideEffect.send(AntiCensorshipSideEffect.ShowToast.GenericError)
            }
        }
    }
}

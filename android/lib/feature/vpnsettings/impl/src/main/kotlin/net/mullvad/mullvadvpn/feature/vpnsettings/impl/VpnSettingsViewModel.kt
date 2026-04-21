package net.mullvad.mullvadvpn.feature.vpnsettings.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.feature.vpnsettings.api.VpnSettingsNavKey
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.common.util.deviceIpVersion
import net.mullvad.mullvadvpn.lib.common.util.quantumResistant
import net.mullvad.mullvadvpn.lib.common.util.selectedObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.repository.AutoStartAndConnectOnBootRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.lib.usecase.SystemVpnSettingsAvailableUseCase

sealed interface VpnSettingsSideEffect {
    sealed interface ShowToast : VpnSettingsSideEffect {
        data object GenericError : ShowToast
    }
}

class VpnSettingsViewModel(
    private val navArgs: VpnSettingsNavKey,
    private val settingsRepository: SettingsRepository,
    private val systemVpnSettingsUseCase: SystemVpnSettingsAvailableUseCase,
    private val autoStartAndConnectOnBootRepository: AutoStartAndConnectOnBootRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {

    private val _uiSideEffect = Channel<VpnSettingsSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()
    val uiState =
        combine(
                settingsRepository.settingsUpdates.filterNotNull(),
                autoStartAndConnectOnBootRepository.autoStartAndConnectOnBoot,
            ) { settings, autoStartAndConnectOnBoot ->
                VpnSettingsUiState.from(
                        mtu = settings.tunnelOptions.mtu,
                        isLocalNetworkSharingEnabled = settings.allowLan,
                        obfuscationMode = settings.selectedObfuscationMode(),
                        quantumResistant = settings.quantumResistant(),
                        systemVpnSettingsAvailable = systemVpnSettingsUseCase(),
                        autoStartAndConnectOnBoot = autoStartAndConnectOnBoot,
                        deviceIpVersion = settings.deviceIpVersion(),
                        isIpv6Enabled = settings.tunnelOptions.enableIpv6,
                        isModal = navArgs.isModal,
                    )
                    .toLc<Boolean, VpnSettingsUiState>()
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(navArgs.isModal),
            )

    fun onToggleLocalNetworkSharing(isEnabled: Boolean) {
        viewModelScope.launch(dispatcher) {
            settingsRepository.setLocalNetworkSharing(isEnabled).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }
    }

    fun onSelectQuantumResistanceSetting(enable: Boolean) {
        viewModelScope.launch(dispatcher) {
            settingsRepository
                .setWireguardQuantumResistant(
                    if (enable) {
                        QuantumResistantState.On
                    } else {
                        QuantumResistantState.Off
                    }
                )
                .onLeft { _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError) }
        }
    }

    fun onToggleAutoStartAndConnectOnBoot(autoStartAndConnect: Boolean) =
        viewModelScope.launch(dispatcher) {
            autoStartAndConnectOnBootRepository.setAutoStartAndConnectOnBoot(autoStartAndConnect)
        }

    fun onDeviceIpVersionSelected(ipVersion: Constraint<IpVersion>) =
        viewModelScope.launch(dispatcher) {
            wireguardConstraintsRepository.setDeviceIpVersion(ipVersion).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }

    fun setIpv6Enabled(enable: Boolean) =
        viewModelScope.launch(dispatcher) {
            settingsRepository.setIpv6Enabled(enable).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }

    fun showGenericErrorToast() = viewModelScope.launch {
        _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
    }
}

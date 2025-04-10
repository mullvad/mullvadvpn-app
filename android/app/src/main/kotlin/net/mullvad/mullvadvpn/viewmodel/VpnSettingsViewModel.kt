package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.None
import arrow.core.Option
import arrow.core.Some
import co.touchlab.kermit.Logger
import com.ramcosta.composedestinations.generated.destinations.VpnSettingsDestination
import java.net.Inet6Address
import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.constant.WIREGUARD_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.AutoStartAndConnectOnBootRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.usecase.SystemVpnSettingsAvailableUseCase
import net.mullvad.mullvadvpn.util.onFirst

sealed interface VpnSettingsSideEffect {
    sealed interface ShowToast : VpnSettingsSideEffect {
        data object ApplySettingsWarning : ShowToast

        data object GenericError : ShowToast
    }

    data object NavigateToDnsDialog : VpnSettingsSideEffect
}

@Suppress("TooManyFunctions")
class VpnSettingsViewModel(
    private val settingsRepository: SettingsRepository,
    relayListRepository: RelayListRepository,
    private val systemVpnSettingsUseCase: SystemVpnSettingsAvailableUseCase,
    private val autoStartAndConnectOnBootRepository: AutoStartAndConnectOnBootRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    savedStateHandle: SavedStateHandle,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {
    private val navArgs = VpnSettingsDestination.argsFrom(savedStateHandle)
    private val _mutableIsContentBlockersExpanded = MutableStateFlow<Option<Boolean>>(None)

    private val _uiSideEffect = Channel<VpnSettingsSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val customPort = MutableStateFlow<Option<Port?>>(None)

    val uiState =
        combine(
                settingsRepository.settingsUpdates.filterNotNull().onFirst {
                    // Initialize wg port and content blockers state expand state
                    val initialPort = it.getWireguardPort().getOrNull()
                    customPort.value =
                        Some(
                            if (initialPort !in WIREGUARD_PRESET_PORTS) {
                                initialPort
                            } else {
                                null
                            }
                        )
                    _mutableIsContentBlockersExpanded.value =
                        Some(it.contentBlockersSettings().isAnyBlockerEnabled())
                },
                relayListRepository.portRanges,
                customPort.filterIsInstance<Some<Port?>>().map { it.value },
                autoStartAndConnectOnBootRepository.autoStartAndConnectOnBoot,
                _mutableIsContentBlockersExpanded.filterIsInstance<Some<Boolean>>().map { it.value },
            ) {
                settings,
                portRanges,
                customWgPort,
                autoStartAndConnectOnBoot,
                isContentBlockersExpanded ->
                VpnSettingsUiState.Content.from(
                    mtu = settings.tunnelOptions.wireguard.mtu,
                    isLocalNetworkSharingEnabled = settings.allowLan,
                    isCustomDnsEnabled = settings.isCustomDnsEnabled(),
                    customDnsItems = settings.addresses().asStringAddressList(),
                    contentBlockersOptions = settings.contentBlockersSettings(),
                    obfuscationMode = settings.selectedObfuscationMode(),
                    selectedUdp2TcpObfuscationPort = settings.obfuscationSettings.udp2tcp.port,
                    selectedShadowsocksObfuscationPort =
                        settings.obfuscationSettings.shadowsocks.port,
                    quantumResistant = settings.quantumResistant(),
                    selectedWireguardPort = settings.getWireguardPort(),
                    customWireguardPort = customWgPort,
                    availablePortRanges = portRanges,
                    systemVpnSettingsAvailable = systemVpnSettingsUseCase(),
                    autoStartAndConnectOnBoot = autoStartAndConnectOnBoot,
                    deviceIpVersion = settings.getDeviceIpVersion(),
                    isIpv6Enabled = settings.tunnelOptions.genericOptions.enableIpv6,
                    isContentBlockersExpanded = isContentBlockersExpanded,
                    isModal = navArgs.isModal,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                VpnSettingsUiState.Loading(navArgs.isModal),
            )

    fun onToggleLocalNetworkSharing(isEnabled: Boolean) {
        viewModelScope.launch(dispatcher) {
            settingsRepository.setLocalNetworkSharing(isEnabled).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }
    }

    fun onToggleCustomDns(enable: Boolean) =
        viewModelScope.launch {
            val settings = settingsRepository.settingsUpdates.value
            if (settings == null) {
                showGenericErrorToast()
                return@launch
            }

            val hasDnsEntries = settings.addresses().isNotEmpty()

            if (hasDnsEntries) {
                settingsRepository
                    .setDnsState(if (enable) DnsState.Custom else DnsState.Default)
                    .fold({ showGenericErrorToast() }, { showApplySettingChangesWarningToast() })
            } else {
                // If they enable custom DNS and has no current entries we show the dialog
                // to add one.
                viewModelScope.launch {
                    _uiSideEffect.send(VpnSettingsSideEffect.NavigateToDnsDialog)
                }
            }
        }

    fun onToggleContentBlockersExpand() =
        _mutableIsContentBlockersExpanded.update { it.map { expand -> !expand } }

    fun onToggleBlockAds(isEnabled: Boolean) = updateContentBlockersAndNotify {
        it.copy(blockAds = isEnabled)
    }

    fun onToggleBlockTrackers(isEnabled: Boolean) = updateContentBlockersAndNotify {
        it.copy(blockTrackers = isEnabled)
    }

    fun onToggleBlockMalware(isEnabled: Boolean) = updateContentBlockersAndNotify {
        it.copy(blockMalware = isEnabled)
    }

    fun onToggleBlockAdultContent(isEnabled: Boolean) = updateContentBlockersAndNotify {
        it.copy(blockAdultContent = isEnabled)
    }

    fun onToggleBlockGambling(isEnabled: Boolean) = updateContentBlockersAndNotify {
        it.copy(blockGambling = isEnabled)
    }

    fun onToggleBlockSocialMedia(isEnabled: Boolean) = updateContentBlockersAndNotify {
        it.copy(blockSocialMedia = isEnabled)
    }

    fun onSelectObfuscationMode(obfuscationMode: ObfuscationMode) {
        viewModelScope.launch(dispatcher) {
            settingsRepository.setObfuscation(obfuscationMode).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }
    }

    fun onObfuscationPortSelected(port: Constraint<Port>) {
        viewModelScope.launch { settingsRepository.setCustomUdp2TcpObfuscationPort(port) }
    }

    fun onSelectQuantumResistanceSetting(quantumResistant: QuantumResistantState) {
        viewModelScope.launch(dispatcher) {
            settingsRepository.setWireguardQuantumResistant(quantumResistant).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }
    }

    fun onWireguardPortSelected(port: Constraint<Port>) {
        if (port is Constraint.Only && port.value !in WIREGUARD_PRESET_PORTS) {
            customPort.update { Some(port.value) }
        }
        viewModelScope.launch {
            wireguardConstraintsRepository.setWireguardPort(port = port).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }
    }

    fun resetCustomPort() {
        customPort.update { Some(null) }
        viewModelScope.launch {
            wireguardConstraintsRepository.setWireguardPort(port = Constraint.Any)
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

    private fun updateContentBlockersAndNotify(update: (DefaultDnsOptions) -> DefaultDnsOptions) =
        viewModelScope.launch(dispatcher) {
            settingsRepository
                .updateContentBlockers(update)
                .fold(
                    {
                        Logger.e("Failed to update content blockers")
                        _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
                    },
                    { showApplySettingChangesWarningToast() },
                )
        }

    private fun List<InetAddress>.asStringAddressList(): List<CustomDnsItem> = map {
        CustomDnsItem(
            address = it.hostAddress ?: EMPTY_STRING,
            isLocal = it.isLocalAddress(),
            isIpv6 = it is Inet6Address,
        )
    }

    private fun Settings.quantumResistant() = tunnelOptions.wireguard.quantumResistant

    private fun Settings.isCustomDnsEnabled() = tunnelOptions.dnsOptions.state == DnsState.Custom

    private fun Settings.addresses() = tunnelOptions.dnsOptions.customOptions.addresses

    private fun Settings.contentBlockersSettings() = tunnelOptions.dnsOptions.defaultOptions

    private fun Settings.selectedObfuscationMode() = obfuscationSettings.selectedObfuscationMode

    private fun Settings.getWireguardPort() =
        relaySettings.relayConstraints.wireguardConstraints.port

    private fun Settings.getDeviceIpVersion() =
        relaySettings.relayConstraints.wireguardConstraints.ipVersion

    private fun InetAddress.isLocalAddress(): Boolean = isLinkLocalAddress || isSiteLocalAddress

    fun showApplySettingChangesWarningToast() =
        viewModelScope.launch {
            _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.ApplySettingsWarning)
        }

    fun showGenericErrorToast() =
        viewModelScope.launch { _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError) }

    companion object {
        private const val EMPTY_STRING = ""
    }
}

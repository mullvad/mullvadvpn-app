package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import java.net.InetAddress
import java.net.UnknownHostException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
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

sealed interface VpnSettingsSideEffect {
    sealed interface ShowToast : VpnSettingsSideEffect {
        data object ApplySettingsWarning : ShowToast

        data object GenericError : ShowToast
    }

    data object NavigateToDnsDialog : VpnSettingsSideEffect
}

@Suppress("TooManyFunctions")
class VpnSettingsViewModel(
    private val repository: SettingsRepository,
    relayListRepository: RelayListRepository,
    private val systemVpnSettingsUseCase: SystemVpnSettingsAvailableUseCase,
    private val autoStartAndConnectOnBootRepository: AutoStartAndConnectOnBootRepository,
    private val wireguardConstraintsRepository: WireguardConstraintsRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {

    private val _uiSideEffect = Channel<VpnSettingsSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val customPort = MutableStateFlow<Port?>(null)

    private val vmState =
        combine(
                repository.settingsUpdates,
                relayListRepository.portRanges,
                customPort,
                autoStartAndConnectOnBootRepository.autoStartAndConnectOnBoot,
            ) { settings, portRanges, customWgPort, autoStartAndConnectOnBoot ->
                VpnSettingsViewModelState(
                    mtuValue = settings?.tunnelOptions?.wireguard?.mtu,
                    isLocalNetworkSharingEnabled = settings?.allowLan == true,
                    isCustomDnsEnabled = settings?.isCustomDnsEnabled() == true,
                    customDnsList = settings?.addresses()?.asStringAddressList() ?: listOf(),
                    contentBlockersOptions =
                        settings?.contentBlockersSettings() ?: DefaultDnsOptions(),
                    obfuscationMode = settings?.selectedObfuscationMode() ?: ObfuscationMode.Off,
                    selectedUdp2TcpObfuscationPort =
                        settings?.obfuscationSettings?.udp2tcp?.port ?: Constraint.Any,
                    selectedShadowsocksObfuscationPort =
                        settings?.obfuscationSettings?.shadowsocks?.port ?: Constraint.Any,
                    quantumResistant = settings?.quantumResistant() ?: QuantumResistantState.Off,
                    selectedWireguardPort = settings?.getWireguardPort() ?: Constraint.Any,
                    customWireguardPort = customWgPort,
                    availablePortRanges = portRanges,
                    systemVpnSettingsAvailable = systemVpnSettingsUseCase(),
                    autoStartAndConnectOnBoot = autoStartAndConnectOnBoot,
                    deviceIpVersion = settings?.getDeviceIpVersion() ?: Constraint.Any,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                VpnSettingsViewModelState.default(),
            )

    val uiState =
        vmState
            .map(VpnSettingsViewModelState::toUiState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                VpnSettingsUiState.createDefault(),
            )

    init {
        viewModelScope.launch(dispatcher) {
            val initialSettings = repository.settingsUpdates.filterNotNull().first()
            customPort.update {
                val initialPort = initialSettings.getWireguardPort()
                if (initialPort.getOrNull() !in WIREGUARD_PRESET_PORTS) {
                    initialPort.getOrNull()
                } else {
                    null
                }
            }
        }
    }

    fun onToggleLocalNetworkSharing(isEnabled: Boolean) {
        viewModelScope.launch(dispatcher) {
            repository.setLocalNetworkSharing(isEnabled).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }
    }

    fun onDnsDialogDismissed() {
        if (vmState.value.customDnsList.isEmpty()) {
            onToggleCustomDns(enable = false)
        }
    }

    fun onToggleCustomDns(enable: Boolean) {
        viewModelScope.launch {
            repository
                .setDnsState(if (enable) DnsState.Custom else DnsState.Default)
                .fold(
                    { _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError) },
                    {
                        if (enable && vmState.value.customDnsList.isEmpty()) {
                            viewModelScope.launch {
                                _uiSideEffect.send(VpnSettingsSideEffect.NavigateToDnsDialog)
                            }
                        } else if (vmState.value.customDnsList.isNotEmpty()) {
                            showApplySettingChangesWarningToast()
                        }
                    },
                )
        }
    }

    fun onToggleBlockAds(isEnabled: Boolean) {
        updateDefaultDnsOptionsViaRepository(
            vmState.value.contentBlockersOptions.copy(blockAds = isEnabled)
        )
        showApplySettingChangesWarningToast()
    }

    fun onToggleBlockTrackers(isEnabled: Boolean) {
        updateDefaultDnsOptionsViaRepository(
            vmState.value.contentBlockersOptions.copy(blockTrackers = isEnabled)
        )
        showApplySettingChangesWarningToast()
    }

    fun onToggleBlockMalware(isEnabled: Boolean) {
        updateDefaultDnsOptionsViaRepository(
            vmState.value.contentBlockersOptions.copy(blockMalware = isEnabled)
        )
        showApplySettingChangesWarningToast()
    }

    fun onToggleBlockAdultContent(isEnabled: Boolean) {
        updateDefaultDnsOptionsViaRepository(
            vmState.value.contentBlockersOptions.copy(blockAdultContent = isEnabled)
        )
        showApplySettingChangesWarningToast()
    }

    fun onToggleBlockGambling(isEnabled: Boolean) {
        updateDefaultDnsOptionsViaRepository(
            vmState.value.contentBlockersOptions.copy(blockGambling = isEnabled)
        )
        showApplySettingChangesWarningToast()
    }

    fun onToggleBlockSocialMedia(isEnabled: Boolean) {
        updateDefaultDnsOptionsViaRepository(
            vmState.value.contentBlockersOptions.copy(blockSocialMedia = isEnabled)
        )
        showApplySettingChangesWarningToast()
    }

    fun onStopEvent() {
        viewModelScope.launch {
            if (vmState.value.customDnsList.isEmpty()) {
                repository.setDnsState(DnsState.Default).onLeft {
                    _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
                }
            }
        }
    }

    fun onSelectObfuscationMode(obfuscationMode: ObfuscationMode) {
        viewModelScope.launch(dispatcher) {
            repository.setObfuscation(obfuscationMode).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }
    }

    fun onObfuscationPortSelected(port: Constraint<Port>) {
        viewModelScope.launch { repository.setCustomUdp2TcpObfuscationPort(port) }
    }

    fun onSelectQuantumResistanceSetting(quantumResistant: QuantumResistantState) {
        viewModelScope.launch(dispatcher) {
            repository.setWireguardQuantumResistant(quantumResistant).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }
    }

    fun onWireguardPortSelected(port: Constraint<Port>) {
        if (port is Constraint.Only && port.value !in WIREGUARD_PRESET_PORTS) {
            customPort.update { port.value }
        }
        viewModelScope.launch { wireguardConstraintsRepository.setWireguardPort(port = port) }
    }

    fun resetCustomPort() {
        val isCustom = vmState.value.isCustomWireguardPort
        customPort.update { null }
        // If custom port was selected, update selection to be any.
        if (isCustom) {
            viewModelScope.launch {
                wireguardConstraintsRepository.setWireguardPort(port = Constraint.Any)
            }
        }
    }

    fun onToggleAutoStartAndConnectOnBoot(autoStartAndConnect: Boolean) {
        viewModelScope.launch(dispatcher) {
            autoStartAndConnectOnBootRepository.setAutoStartAndConnectOnBoot(autoStartAndConnect)
        }
    }

    fun onDeviceIpVersionSelected(ipVersion: Constraint<IpVersion>) {
        viewModelScope.launch(dispatcher) {
            wireguardConstraintsRepository.setDeviceIpVersion(ipVersion).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }
    }

    private fun updateDefaultDnsOptionsViaRepository(contentBlockersOption: DefaultDnsOptions) =
        viewModelScope.launch(dispatcher) {
            repository
                .setDnsOptions(
                    isCustomDnsEnabled = vmState.value.isCustomDnsEnabled,
                    dnsList = vmState.value.customDnsList.map { it.address }.asInetAddressList(),
                    contentBlockersOptions = contentBlockersOption,
                )
                .onLeft { _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError) }
        }

    private fun List<String>.asInetAddressList(): List<InetAddress> {
        return try {
            map { InetAddress.getByName(it) }
        } catch (_: UnknownHostException) {
            Logger.e("Error parsing the DNS address list.")
            emptyList()
        }
    }

    private fun List<InetAddress>.asStringAddressList(): List<CustomDnsItem> {
        return map {
            CustomDnsItem(address = it.hostAddress ?: EMPTY_STRING, isLocal = it.isLocalAddress())
        }
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

    private fun InetAddress.isLocalAddress(): Boolean {
        return isLinkLocalAddress || isSiteLocalAddress
    }

    fun showApplySettingChangesWarningToast() {
        viewModelScope.launch {
            _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.ApplySettingsWarning)
        }
    }

    fun showGenericErrorToast() {
        viewModelScope.launch { _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError) }
    }

    companion object {
        private const val EMPTY_STRING = ""
    }
}

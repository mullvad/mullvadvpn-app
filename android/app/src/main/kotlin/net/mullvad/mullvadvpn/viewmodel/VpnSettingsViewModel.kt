package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import java.net.Inet6Address
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
import net.mullvad.mullvadvpn.compose.state.VpnSettingItem
import net.mullvad.mullvadvpn.constant.WIREGUARD_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
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

    private val _mutableIsContentBlockersExpanded = MutableStateFlow(false)

    private val _uiSideEffect = Channel<VpnSettingsSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val customPort = MutableStateFlow<Port?>(null)

    private val vmState =
        combine(
                repository.settingsUpdates,
                relayListRepository.portRanges,
                customPort,
                autoStartAndConnectOnBootRepository.autoStartAndConnectOnBoot,
                _mutableIsContentBlockersExpanded,
            ) {
                settings,
                portRanges,
                customWgPort,
                autoStartAndConnectOnBoot,
                isContentBlockersExpanded ->
                VpnSettingsViewModelState(
                    mtu = settings?.tunnelOptions?.wireguard?.mtu,
                    isLocalNetworkSharingEnabled = settings?.allowLan == true,
                    isCustomDnsEnabled = settings?.isCustomDnsEnabled() == true,
                    customDnsItems = settings?.addresses()?.asStringAddressList() ?: listOf(),
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
                    isIpv6Enabled = settings?.tunnelOptions?.genericOptions?.enableIpv6 == true,
                    isContentBlockersExpanded = isContentBlockersExpanded,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                VpnSettingsViewModelState.default(),
            )

    val uiState =
        vmState
            .map {
                VpnSettingsUiState.Content(
                    with(it) {
                        createSettingsList(
                            mtu,
                            isLocalNetworkSharingEnabled,
                            isCustomDnsEnabled,
                            customDnsItems,
                            contentBlockersOptions,
                            obfuscationMode,
                            selectedUdp2TcpObfuscationPort,
                            selectedShadowsocksObfuscationPort,
                            quantumResistant,
                            selectedWireguardPort,
                            customWireguardPort,
                            availablePortRanges,
                            systemVpnSettingsAvailable,
                            autoStartAndConnectOnBoot,
                            deviceIpVersion,
                            isIpv6Enabled,
                            isContentBlockersExpanded,
                        )
                    }
                )
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), VpnSettingsUiState.Loading)

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
        if (vmState.value.customDnsItems.isEmpty()) {
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
                        if (enable && vmState.value.customDnsItems.isEmpty()) {
                            viewModelScope.launch {
                                _uiSideEffect.send(VpnSettingsSideEffect.NavigateToDnsDialog)
                            }
                        } else if (vmState.value.customDnsItems.isNotEmpty()) {
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
            if (vmState.value.customDnsItems.isEmpty()) {
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

    fun setIpv6Enabled(enable: Boolean) {
        viewModelScope.launch(dispatcher) {
            repository.setIpv6Enabled(enable).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }
    }

    private fun updateDefaultDnsOptionsViaRepository(contentBlockersOption: DefaultDnsOptions) =
        viewModelScope.launch(dispatcher) {
            repository
                .setDnsOptions(
                    isCustomDnsEnabled = vmState.value.isCustomDnsEnabled,
                    dnsList = vmState.value.customDnsItems.map { it.address }.asInetAddressList(),
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
            CustomDnsItem(
                address = it.hostAddress ?: EMPTY_STRING,
                isLocal = it.isLocalAddress(),
                isIpv6 = it is Inet6Address,
            )
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

    private fun createSettingsList(
        mtu: Mtu?,
        isLocalNetworkSharingEnabled: Boolean,
        isCustomDnsEnabled: Boolean,
        customDnsItems: List<CustomDnsItem>,
        contentBlockersOptions: DefaultDnsOptions,
        obfuscationMode: ObfuscationMode,
        selectedUdp2TcpObfuscationPort: Constraint<Port>,
        selectedShadowssocksObfuscationPort: Constraint<Port>,
        quantumResistant: QuantumResistantState,
        selectedWireguardPort: Constraint<Port>,
        customWireguardPort: Port?,
        availablePortRanges: List<PortRange>,
        systemVpnSettingsAvailable: Boolean,
        autoStartAndConnectOnBoot: Boolean,
        deviceIpVersion: Constraint<IpVersion>,
        isIpv6Enabled: Boolean,
        isContentBlockersExpanded: Boolean,
    ): List<VpnSettingItem> =
        buildList<VpnSettingItem> {
            if (systemVpnSettingsAvailable) {
                add(VpnSettingItem.AutoConnectAndLockdownModeHeader)
                add(VpnSettingItem.AutoConnectAndLockdownModeInfo)
            } else {
                add(VpnSettingItem.ConnectDeviceOnStartUpHeader(autoStartAndConnectOnBoot))
                add(VpnSettingItem.ConnectDeviceOnStartUpInfo)
            }

            // Local network sharing
            add(VpnSettingItem.LocalNetworkSharingHeader(isLocalNetworkSharingEnabled))
            add(VpnSettingItem.Spacer)

            // Dns Content Blockers
            add(VpnSettingItem.DnsContentBlockers(isCustomDnsEnabled, isContentBlockersExpanded))
            add(VpnSettingItem.Divider)

            if (isContentBlockersExpanded) {
                with(contentBlockersOptions) {
                    add(VpnSettingItem.DnsContentBlockerItem.Ads(blockAds))
                    add(VpnSettingItem.Divider)
                    add(VpnSettingItem.DnsContentBlockerItem.Trackers(blockTrackers))
                    add(VpnSettingItem.Divider)
                    add(VpnSettingItem.DnsContentBlockerItem.Malware(blockMalware))
                    add(VpnSettingItem.Divider)
                    add(VpnSettingItem.DnsContentBlockerItem.Gambling(blockGambling))
                    add(VpnSettingItem.Divider)
                    add(VpnSettingItem.DnsContentBlockerItem.AdultContent(blockAdultContent))
                    add(VpnSettingItem.Divider)
                    add(VpnSettingItem.DnsContentBlockerItem.SocialMedia(blockSocialMedia))
                }
                if (isCustomDnsEnabled) {
                    add(VpnSettingItem.DnsContentBlockersUnavailable)
                }
            }

            // Custom DNS
            add(
                VpnSettingItem.CustomDnsServerHeader(
                    isCustomDnsEnabled,
                    !contentBlockersOptions.isAnyBlockerEnabled(),
                )
            )
            if (isCustomDnsEnabled) {
                customDnsItems.forEachIndexed { index, item ->
                    add(
                        VpnSettingItem.CustomDnsEntry(
                            index,
                            item,
                            showUnreachableLocalDnsWarning =
                                item.isLocal && !isLocalNetworkSharingEnabled,
                            showUnreachableIpv6DnsWarning = item.isIpv6 && !isIpv6Enabled,
                        )
                    )
                    add(VpnSettingItem.Divider)
                }
                if (customDnsItems.isNotEmpty()) {
                    add(VpnSettingItem.CustomDnsAdd)
                }
            }

            if (contentBlockersOptions.isAnyBlockerEnabled()) {
                add(VpnSettingItem.CustomDnsUnavailable)
            } else {
                add(VpnSettingItem.CustomDnsInfo)
            }

            add(VpnSettingItem.Spacer)

            // Wireguard Port
            val isWireguardPortEnabled =
                obfuscationMode == ObfuscationMode.Auto || obfuscationMode == ObfuscationMode.Off
            add(VpnSettingItem.WireguardPortHeader(isWireguardPortEnabled, availablePortRanges))
            add(VpnSettingItem.Divider)

            add(
                VpnSettingItem.WireguardPortItem.Automatic(
                    isWireguardPortEnabled,
                    selectedWireguardPort == Constraint.Any,
                )
            )
            add(VpnSettingItem.Divider)

            WIREGUARD_PRESET_PORTS.forEach { port ->
                add(
                    VpnSettingItem.WireguardPortItem.FixedPort(
                        isWireguardPortEnabled,
                        selectedWireguardPort.getOrNull() == port,
                        port,
                    )
                )
                add(VpnSettingItem.Divider)
            }
            add(
                VpnSettingItem.WireguardPortItem.WireguardPortCustom(
                    isWireguardPortEnabled,
                    selectedWireguardPort is Constraint.Only &&
                        selectedWireguardPort.value == customWireguardPort,
                    customWireguardPort,
                )
            )

            if (!isWireguardPortEnabled) {
                add(VpnSettingItem.WireguardPortUnavailable)
            }

            add(VpnSettingItem.Spacer)

            // Wireguard Obfuscation
            add(VpnSettingItem.ObfuscationHeader)
            add(VpnSettingItem.Divider)
            add(VpnSettingItem.ObfuscationItem.Automatic(obfuscationMode == ObfuscationMode.Auto))
            add(VpnSettingItem.Divider)
            add(
                VpnSettingItem.ObfuscationItem.Shadowsocks(
                    obfuscationMode == ObfuscationMode.Shadowsocks,
                    selectedShadowssocksObfuscationPort,
                )
            )
            add(VpnSettingItem.Divider)
            add(
                VpnSettingItem.ObfuscationItem.UdpOverTcp(
                    obfuscationMode == ObfuscationMode.Auto,
                    selectedUdp2TcpObfuscationPort,
                )
            )
            add(VpnSettingItem.Divider)
            add(VpnSettingItem.ObfuscationItem.Automatic(obfuscationMode == ObfuscationMode.Off))

            add(VpnSettingItem.Spacer)

            // Quantum Resistance
            add(VpnSettingItem.QuantumResistanceHeader)
            add(VpnSettingItem.Divider)
            add(
                VpnSettingItem.QuantumItem.Automatic(quantumResistant == QuantumResistantState.Auto)
            )
            add(VpnSettingItem.Divider)
            add(VpnSettingItem.QuantumItem.On(quantumResistant == QuantumResistantState.On))
            add(VpnSettingItem.Divider)
            add(VpnSettingItem.QuantumItem.Off(quantumResistant == QuantumResistantState.Off))

            add(VpnSettingItem.Spacer)

            // Device Ip Version
            add(VpnSettingItem.DeviceIpVersionHeader)
            add(VpnSettingItem.Divider)
            add(VpnSettingItem.DeviceIpVersionItem.Automatic(deviceIpVersion == Constraint.Any))
            add(VpnSettingItem.Divider)
            add(
                VpnSettingItem.DeviceIpVersionItem.Ip(
                    deviceIpVersion.getOrNull() == IpVersion.IPV4,
                    IpVersion.IPV4,
                )
            )
            add(VpnSettingItem.Divider)
            add(
                VpnSettingItem.DeviceIpVersionItem.Ip(
                    deviceIpVersion.getOrNull() == IpVersion.IPV6,
                    IpVersion.IPV6,
                )
            )

            add(VpnSettingItem.Spacer)

            // IPv6
            add(VpnSettingItem.EnableIpv6Header(isIpv6Enabled))

            add(VpnSettingItem.Spacer)

            // MTU
            add(VpnSettingItem.MtuHeader(mtu))
            add(VpnSettingItem.MtuInfo)

            add(VpnSettingItem.ServerIpOverridesHeader)
        }

    companion object {
        private const val EMPTY_STRING = ""
    }
}

package net.mullvad.mullvadvpn.viewmodel

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
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
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.DnsState
import net.mullvad.mullvadvpn.model.ObfuscationSettings
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.SelectedObfuscation
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.Udp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.model.WireguardConstraints
import net.mullvad.mullvadvpn.model.relayConstraints
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.PortRangeUseCase
import net.mullvad.mullvadvpn.usecase.SystemVpnSettingsUseCase
import net.mullvad.mullvadvpn.util.isCustom

sealed interface VpnSettingsSideEffect {
    sealed interface ShowToast : VpnSettingsSideEffect {
        data object ApplySettingsWarning : ShowToast

        data object GenericError : ShowToast
    }

    data object NavigateToDnsDialog : VpnSettingsSideEffect
}

class VpnSettingsViewModel(
    private val repository: SettingsRepository,
    portRangeUseCase: PortRangeUseCase,
    private val relayListRepository: RelayListRepository,
    private val systemVpnSettingsUseCase: SystemVpnSettingsUseCase,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {

    private val _uiSideEffect = Channel<VpnSettingsSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val customPort = MutableStateFlow<Constraint<Port>?>(null)

    private val vmState =
        combine(repository.settingsUpdates, portRangeUseCase.portRanges(), customPort) {
                settings,
                portRanges,
                customWgPort ->
                VpnSettingsViewModelState(
                    mtuValue = settings?.mtuString() ?: "",
                    isAutoConnectEnabled = settings?.autoConnect ?: false,
                    isLocalNetworkSharingEnabled = settings?.allowLan ?: false,
                    isCustomDnsEnabled = settings?.isCustomDnsEnabled() ?: false,
                    customDnsList = settings?.addresses()?.asStringAddressList() ?: listOf(),
                    contentBlockersOptions =
                        settings?.contentBlockersSettings() ?: DefaultDnsOptions(),
                    selectedObfuscation =
                        settings?.selectedObfuscationSettings() ?: SelectedObfuscation.Off,
                    quantumResistant = settings?.quantumResistant() ?: QuantumResistantState.Off,
                    selectedWireguardPort = settings?.getWireguardPort() ?: Constraint.Any,
                    customWireguardPort = customWgPort,
                    availablePortRanges = portRanges,
                    systemVpnSettingsAvailable =
                        systemVpnSettingsUseCase.systemVpnSettingsAvailable()
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                VpnSettingsViewModelState.default()
            )

    val uiState =
        vmState
            .map(VpnSettingsViewModelState::toUiState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                VpnSettingsUiState.createDefault()
            )

    init {
        viewModelScope.launch(dispatcher) {
            val initialSettings = repository.settingsUpdates.filterNotNull().first()
            customPort.update {
                val initialPort = initialSettings.getWireguardPort()
                if (initialPort.isCustom()) {
                    initialPort
                } else {
                    null
                }
            }
        }
    }

    fun onToggleAutoConnect(isEnabled: Boolean) {
        viewModelScope.launch(dispatcher) {
            repository.setAutoConnect(isEnabled).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
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
                    }
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

    fun onSelectObfuscationSetting(selectedObfuscation: SelectedObfuscation) {
        viewModelScope.launch(dispatcher) {
            repository
                .setObfuscationOptions(
                    ObfuscationSettings(
                        selectedObfuscation = selectedObfuscation,
                        udp2tcp = Udp2TcpObfuscationSettings(Constraint.Any)
                    )
                )
                .onLeft { _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError) }
        }
    }

    fun onSelectQuantumResistanceSetting(quantumResistant: QuantumResistantState) {
        viewModelScope.launch(dispatcher) {
            repository.setWireguardQuantumResistant(quantumResistant).onLeft {
                _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError)
            }
        }
    }

    fun onWireguardPortSelected(port: Constraint<Port>) {
        if (port.isCustom()) {
            customPort.update { port }
        }
        viewModelScope.launch {
            relayListRepository.updateSelectedWireguardConstraints(
                WireguardConstraints(port = port)
            )
        }
    }

    fun resetCustomPort() {
        customPort.update { null }
        // If custom port was selected, update selection to be any.
        if (vmState.value.selectedWireguardPort.isCustom()) {
            viewModelScope.launch {
                relayListRepository.updateSelectedWireguardConstraints(
                    WireguardConstraints(port = Constraint.Any)
                )
            }
        }
    }

    private fun updateDefaultDnsOptionsViaRepository(contentBlockersOption: DefaultDnsOptions) =
        viewModelScope.launch(dispatcher) {
            repository
                .setDnsOptions(
                    isCustomDnsEnabled = vmState.value.isCustomDnsEnabled,
                    dnsList = vmState.value.customDnsList.map { it.address }.asInetAddressList(),
                    contentBlockersOptions = contentBlockersOption
                )
                .onLeft { _uiSideEffect.send(VpnSettingsSideEffect.ShowToast.GenericError) }
        }

    private fun List<String>.asInetAddressList(): List<InetAddress> {
        return try {
            map { InetAddress.getByName(it) }
        } catch (ex: UnknownHostException) {
            Log.e("mullvad", "Error parsing the DNS address list.")
            emptyList()
        }
    }

    private fun List<InetAddress>.asStringAddressList(): List<CustomDnsItem> {
        return map {
            CustomDnsItem(address = it.hostAddress ?: EMPTY_STRING, isLocal = it.isLocalAddress())
        }
    }

    private fun Settings.mtuString() = tunnelOptions.wireguard.mtu?.toString() ?: EMPTY_STRING

    private fun Settings.quantumResistant() = tunnelOptions.wireguard.quantumResistant

    private fun Settings.isCustomDnsEnabled() =
        tunnelOptions.dnsOptions.currentDnsOption == DnsState.Custom

    private fun Settings.addresses() = tunnelOptions.dnsOptions.customOptions.addresses

    private fun Settings.contentBlockersSettings() = tunnelOptions.dnsOptions.defaultOptions

    private fun Settings.selectedObfuscationSettings() = obfuscationSettings.selectedObfuscation

    private fun Settings.getWireguardPort() =
        relaySettings.relayConstraints.wireguardConstraints.port

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

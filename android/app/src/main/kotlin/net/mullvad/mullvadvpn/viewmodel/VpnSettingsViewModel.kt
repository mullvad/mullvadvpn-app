package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.DnsState
import net.mullvad.mullvadvpn.model.ObfuscationSettings
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.SelectedObfuscation
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.Udp2TcpObfuscationSettings
import net.mullvad.mullvadvpn.model.WireguardConstraints
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.PortRangeUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import net.mullvad.mullvadvpn.util.isValidMtu
import org.apache.commons.validator.routines.InetAddressValidator

class VpnSettingsViewModel(
    private val repository: SettingsRepository,
    private val inetAddressValidator: InetAddressValidator,
    private val resources: Resources,
    portRangeUseCase: PortRangeUseCase,
    private val relayListUseCase: RelayListUseCase,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {

    private val _toastMessages = MutableSharedFlow<String>(extraBufferCapacity = 1)
    @Suppress("konsist.ensure public properties use permitted names")
    val toastMessages = _toastMessages.asSharedFlow()

    private val dialogState = MutableStateFlow<VpnSettingsDialogState?>(null)

    private val vmState =
        combine(repository.settingsUpdates, portRangeUseCase.portRanges(), dialogState) {
                settings,
                portRanges,
                dialogState ->
                VpnSettingsViewModelState(
                    mtuValue = settings?.mtuString() ?: "",
                    isAutoConnectEnabled = settings?.autoConnect ?: false,
                    isLocalNetworkSharingEnabled = settings?.allowLan ?: false,
                    isCustomDnsEnabled = settings?.isCustomDnsEnabled() ?: false,
                    customDnsList = settings?.addresses()?.asStringAddressList() ?: listOf(),
                    contentBlockersOptions =
                        settings?.contentBlockersSettings() ?: DefaultDnsOptions(),
                    isAllowLanEnabled = settings?.allowLan ?: false,
                    selectedObfuscation =
                        settings?.selectedObfuscationSettings() ?: SelectedObfuscation.Off,
                    dialogState = dialogState,
                    quantumResistant = settings?.quantumResistant() ?: QuantumResistantState.Off,
                    selectedWireguardPort = settings?.getWireguardPort() ?: Constraint.Any(),
                    availablePortRanges = portRanges
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

    fun onMtuCellClick() {
        dialogState.update { VpnSettingsDialogState.MtuDialog(vmState.value.mtuValue) }
    }

    fun onSaveMtuClick(mtuValue: Int) =
        viewModelScope.launch(dispatcher) {
            if (mtuValue.isValidMtu()) {
                repository.setWireguardMtu(mtuValue)
            }
            hideDialog()
        }

    fun onRestoreMtuClick() =
        viewModelScope.launch(dispatcher) {
            repository.setWireguardMtu(null)
            hideDialog()
        }

    fun onCancelDialogClick() {
        hideDialog()
    }

    fun onLocalNetworkSharingInfoClick() {
        dialogState.update { VpnSettingsDialogState.LocalNetworkSharingInfoDialog }
    }

    fun onContentsBlockerInfoClick() {
        dialogState.update { VpnSettingsDialogState.ContentBlockersInfoDialog }
    }

    fun onCustomDnsInfoClick() {
        dialogState.update { VpnSettingsDialogState.CustomDnsInfoDialog }
    }

    fun onMalwareInfoClick() {
        dialogState.update { VpnSettingsDialogState.MalwareInfoDialog }
    }

    fun onDismissInfoClick() {
        hideDialog()
    }

    fun onDnsClick(index: Int? = null) {
        val stagedDns =
            if (index == null) {
                StagedDns.NewDns(
                    item = CustomDnsItem.default(),
                    validationResult = StagedDns.ValidationResult.InvalidAddress
                )
            } else {
                vmState.value.customDnsList.getOrNull(index)?.let { listItem ->
                    StagedDns.EditDns(item = listItem, index = index)
                }
            }

        if (stagedDns != null) {
            dialogState.update { VpnSettingsDialogState.DnsDialog(stagedDns) }
        }
    }

    fun onDnsInputChange(ipAddress: String) {
        dialogState.update { state ->
            val dialog = state as? VpnSettingsDialogState.DnsDialog ?: return

            val error =
                when {
                    ipAddress.isBlank() || ipAddress.isValidIp().not() -> {
                        StagedDns.ValidationResult.InvalidAddress
                    }
                    ipAddress.isDuplicateDns((state.stagedDns as? StagedDns.EditDns)?.index) -> {
                        StagedDns.ValidationResult.DuplicateAddress
                    }
                    else -> StagedDns.ValidationResult.Success
                }

            return@update VpnSettingsDialogState.DnsDialog(
                stagedDns =
                    if (dialog.stagedDns is StagedDns.EditDns) {
                        StagedDns.EditDns(
                            item =
                                CustomDnsItem(
                                    address = ipAddress,
                                    isLocal = ipAddress.isLocalAddress()
                                ),
                            validationResult = error,
                            index = dialog.stagedDns.index
                        )
                    } else {
                        StagedDns.NewDns(
                            item =
                                CustomDnsItem(
                                    address = ipAddress,
                                    isLocal = ipAddress.isLocalAddress()
                                ),
                            validationResult = error
                        )
                    }
            )
        }
    }

    fun onSaveDnsClick() =
        viewModelScope.launch(dispatcher) {
            val dialog =
                vmState.value.dialogState as? VpnSettingsDialogState.DnsDialog ?: return@launch

            if (dialog.stagedDns.isValid().not()) return@launch

            val updatedList =
                vmState.value.customDnsList
                    .toMutableList()
                    .map { it.address }
                    .toMutableList()
                    .let { activeList ->
                        if (dialog.stagedDns is StagedDns.EditDns) {
                            activeList
                                .apply {
                                    set(dialog.stagedDns.index, dialog.stagedDns.item.address)
                                }
                                .asInetAddressList()
                        } else {
                            activeList
                                .apply { add(dialog.stagedDns.item.address) }
                                .asInetAddressList()
                        }
                    }

            repository.setDnsOptions(
                isCustomDnsEnabled = true,
                dnsList = updatedList,
                contentBlockersOptions = vmState.value.contentBlockersOptions
            )

            hideDialog()
        }

    fun onToggleAutoConnect(isEnabled: Boolean) {
        viewModelScope.launch(dispatcher) { repository.setAutoConnect(isEnabled) }
    }

    fun onToggleLocalNetworkSharing(isEnabled: Boolean) {
        viewModelScope.launch(dispatcher) { repository.setLocalNetworkSharing(isEnabled) }
    }

    fun onToggleDnsClick(isEnabled: Boolean) {
        updateCustomDnsState(isEnabled)
        if (isEnabled && vmState.value.customDnsList.isEmpty()) {
            onDnsClick(null)
        }
        showApplySettingChangesWarningToast()
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

    fun onRemoveDnsClick() =
        viewModelScope.launch(dispatcher) {
            val dialog =
                vmState.value.dialogState as? VpnSettingsDialogState.DnsDialog ?: return@launch

            val updatedList =
                vmState.value.customDnsList
                    .toMutableList()
                    .filter { it.address != dialog.stagedDns.item.address }
                    .map { it.address }
                    .asInetAddressList()

            repository.setDnsOptions(
                isCustomDnsEnabled = vmState.value.isCustomDnsEnabled && updatedList.isNotEmpty(),
                dnsList = updatedList,
                contentBlockersOptions = vmState.value.contentBlockersOptions
            )
            hideDialog()
        }

    fun onStopEvent() {
        if (vmState.value.customDnsList.isEmpty()) {
            updateCustomDnsState(false)
        }
    }

    fun onSelectObfuscationSetting(selectedObfuscation: SelectedObfuscation) {
        viewModelScope.launch(dispatcher) {
            repository.setObfuscationOptions(
                ObfuscationSettings(
                    selectedObfuscation = selectedObfuscation,
                    udp2tcp = Udp2TcpObfuscationSettings(Constraint.Any())
                )
            )
        }
    }

    fun onObfuscationInfoClick() {
        dialogState.update { VpnSettingsDialogState.ObfuscationInfoDialog }
    }

    fun onSelectQuantumResistanceSetting(quantumResistant: QuantumResistantState) {
        viewModelScope.launch(dispatcher) {
            repository.setWireguardQuantumResistant(quantumResistant)
        }
    }

    fun onQuantumResistanceInfoClicked() {
        dialogState.update { VpnSettingsDialogState.QuantumResistanceInfoDialog }
    }

    fun onWireguardPortSelected(port: Constraint<Port>) {
        relayListUseCase.updateSelectedWireguardConstraints(WireguardConstraints(port = port))
        hideDialog()
    }

    fun onWireguardPortInfoClicked() {
        dialogState.update { VpnSettingsDialogState.WireguardPortInfoDialog }
    }

    fun onShowCustomPortDialog() {
        dialogState.update { VpnSettingsDialogState.CustomPortDialog }
    }

    private fun updateDefaultDnsOptionsViaRepository(contentBlockersOption: DefaultDnsOptions) =
        viewModelScope.launch(dispatcher) {
            repository.setDnsOptions(
                isCustomDnsEnabled = vmState.value.isCustomDnsEnabled,
                dnsList = vmState.value.customDnsList.map { it.address }.asInetAddressList(),
                contentBlockersOptions = contentBlockersOption
            )
        }

    private fun hideDialog() {
        dialogState.update { null }
    }

    fun onCancelDns() {
        if (
            vmState.value.dialogState is VpnSettingsDialogState.DnsDialog &&
                vmState.value.customDnsList.isEmpty()
        ) {
            onToggleDnsClick(false)
        }
        hideDialog()
    }

    private fun String.isDuplicateDns(stagedIndex: Int? = null): Boolean {
        return vmState.value.customDnsList
            .filterIndexed { index, listItem -> index != stagedIndex && listItem.address == this }
            .isNotEmpty()
    }

    private fun List<String>.asInetAddressList(): List<InetAddress> {
        return try {
            map { InetAddress.getByName(it) }
        } catch (ex: Exception) {
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

    private fun Settings.isCustomDnsEnabled() = tunnelOptions.dnsOptions.state == DnsState.Custom

    private fun Settings.addresses() = tunnelOptions.dnsOptions.customOptions.addresses

    private fun Settings.contentBlockersSettings() = tunnelOptions.dnsOptions.defaultOptions

    private fun Settings.selectedObfuscationSettings() = obfuscationSettings.selectedObfuscation

    private fun Settings.getWireguardPort() =
        when (relaySettings) {
            RelaySettings.CustomTunnelEndpoint -> Constraint.Any()
            is RelaySettings.Normal ->
                (relaySettings as RelaySettings.Normal).relayConstraints.wireguardConstraints.port
        }

    private fun String.isValidIp(): Boolean {
        return inetAddressValidator.isValid(this)
    }

    private fun String.isLocalAddress(): Boolean {
        return isValidIp() && InetAddress.getByName(this).isLocalAddress()
    }

    private fun InetAddress.isLocalAddress(): Boolean {
        return isLinkLocalAddress || isSiteLocalAddress
    }

    private fun updateCustomDnsState(isEnabled: Boolean) {
        viewModelScope.launch(dispatcher) {
            repository.setDnsOptions(
                isEnabled,
                dnsList = vmState.value.customDnsList.map { it.address }.asInetAddressList(),
                contentBlockersOptions = vmState.value.contentBlockersOptions
            )
        }
    }

    private fun showApplySettingChangesWarningToast() {
        _toastMessages.tryEmit(resources.getString(R.string.settings_changes_effect_warning_short))
    }

    companion object {
        private const val EMPTY_STRING = ""
    }
}

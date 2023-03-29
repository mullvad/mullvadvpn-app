package net.mullvad.mullvadvpn.viewmodel

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.AdvancedSettingsUiState
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.DnsState
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.isValidMtu
import org.apache.commons.validator.routines.InetAddressValidator

class AdvancedSettingsViewModel(
    private val repository: SettingsRepository,
    private val inetAddressValidator: InetAddressValidator,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {

    private val dialogState =
        MutableStateFlow<AdvancedSettingsDialogState>(AdvancedSettingsDialogState.NoDialog)

    private val vmState =
        combine(repository.settingsUpdates, dialogState) { settings, interaction ->
            AdvancedSettingsViewModelState(
                mtuValue = settings?.mtuString() ?: "",
                isCustomDnsEnabled = settings?.isCustomDnsEnabled() ?: false,
                customDnsList = settings?.addresses()?.asStringAddressList() ?: listOf(),
                contentBlockersOptions = settings?.contentBlockersSettings() ?: DefaultDnsOptions(),
                isAllowLanEnabled = settings?.allowLan ?: false,
                dialogState = interaction,
            )
        }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                AdvancedSettingsViewModelState.default(),
            )

    val uiState =
        vmState
            .map(AdvancedSettingsViewModelState::toUiState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                AdvancedSettingsUiState.DefaultUiState(),
            )

    fun onMtuCellClick() {
        dialogState.update { AdvancedSettingsDialogState.MtuDialog(vmState.value.mtuValue) }
    }

    fun onMtuInputChange(value: String) {
        dialogState.update { AdvancedSettingsDialogState.MtuDialog(value) }
    }

    fun onSaveMtuClick() =
        viewModelScope.launch(dispatcher) {
            val dialog = dialogState.value as? AdvancedSettingsDialogState.MtuDialog
            dialog
                ?.mtuEditValue
                ?.toIntOrNull()
                ?.takeIf { it.isValidMtu() }
                ?.let { mtu -> repository.setWireguardMtu(mtu) }
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

    fun onContentsBlockerInfoClick() {
        dialogState.update { AdvancedSettingsDialogState.ContentsBlockerInfoDialog }
    }

    fun onMalwareInfoClick() {
        dialogState.update { AdvancedSettingsDialogState.MalwareInfoDialog }
    }

    fun onDismissInfoClick() {
        hideDialog()
    }

    fun onDnsClick(index: Int? = null) {
        val stagedDns =
            if (index == null) {
                StagedDns.NewDns(CustomDnsItem.default())
            } else {
                vmState.value.customDnsList.getOrNull(index)?.let { listItem ->
                    StagedDns.EditDns(item = listItem, index = index)
                }
            }

        if (stagedDns != null) {
            dialogState.update { AdvancedSettingsDialogState.DnsDialog(stagedDns) }
        }
    }

    fun onDnsInputChange(ipAddress: String) {
        dialogState.update { state ->
            val dialog = state as? AdvancedSettingsDialogState.DnsDialog ?: return

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

            return@update AdvancedSettingsDialogState.DnsDialog(
                stagedDns =
                if (dialog.stagedDns is StagedDns.EditDns) {
                    StagedDns.EditDns(
                        item =
                        CustomDnsItem(
                            address = ipAddress,
                            isLocal = ipAddress.isLocalAddress(),
                        ),
                        validationResult = error,
                        index = dialog.stagedDns.index,
                    )
                } else {
                    StagedDns.NewDns(
                        item =
                        CustomDnsItem(
                            address = ipAddress,
                            isLocal = ipAddress.isLocalAddress(),
                        ),
                        validationResult = error,
                    )
                },
            )
        }
    }

    fun onSaveDnsClick() =
        viewModelScope.launch(dispatcher) {
            val dialog =
                vmState.value.dialogState as? AdvancedSettingsDialogState.DnsDialog ?: return@launch

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

    fun onToggleDnsClick(isEnabled: Boolean) =
        viewModelScope.launch(dispatcher) {
            repository.setDnsOptions(
                isEnabled,
                dnsList = vmState.value.customDnsList.map { it.address }.asInetAddressList(),
                contentBlockersOptions = vmState.value.contentBlockersOptions
            )
        }

    fun onToggleBlockAds(isEnabled: Boolean) {
        var contentBlockerOptions = DefaultDnsOptions(
            blockAds = isEnabled,
            blockTrackers = vmState.value.contentBlockersOptions.blockTrackers,
            blockMalware = vmState.value.contentBlockersOptions.blockMalware,
            blockAdultContent = vmState.value.contentBlockersOptions.blockAdultContent,
            blockGambling = vmState.value.contentBlockersOptions.blockGambling,
        )
        onContentBlockerSettingsChanged(contentBlockerOptions)
    }

    fun onToggleBlockTrackers(isEnabled: Boolean) {
        var contentBlockerOptions = DefaultDnsOptions(
            blockAds = vmState.value.contentBlockersOptions.blockAds,
            blockTrackers = isEnabled,
            blockMalware = vmState.value.contentBlockersOptions.blockMalware,
            blockAdultContent = vmState.value.contentBlockersOptions.blockAdultContent,
            blockGambling = vmState.value.contentBlockersOptions.blockGambling,
        )
        onContentBlockerSettingsChanged(contentBlockerOptions)
    }

    fun onToggleBlockMalware(isEnabled: Boolean) {
        var contentBlockerOptions = DefaultDnsOptions(
            blockAds = vmState.value.contentBlockersOptions.blockAds,
            blockTrackers = vmState.value.contentBlockersOptions.blockTrackers,
            blockMalware = isEnabled,
            blockAdultContent = vmState.value.contentBlockersOptions.blockAdultContent,
            blockGambling = vmState.value.contentBlockersOptions.blockGambling,
        )
        onContentBlockerSettingsChanged(contentBlockerOptions)
    }

    fun onToggleBlockAdultContent(isEnabled: Boolean) {
        var contentBlockerOptions = DefaultDnsOptions(
            blockAds = vmState.value.contentBlockersOptions.blockAds,
            blockTrackers = vmState.value.contentBlockersOptions.blockTrackers,
            blockMalware = vmState.value.contentBlockersOptions.blockMalware,
            blockAdultContent = isEnabled,
            blockGambling = vmState.value.contentBlockersOptions.blockGambling,
        )
        onContentBlockerSettingsChanged(contentBlockerOptions)
    }

    fun onToggleBlockGambling(isEnabled: Boolean) {
        var contentBlockerOptions = DefaultDnsOptions(
            blockAds = vmState.value.contentBlockersOptions.blockAds,
            blockTrackers = vmState.value.contentBlockersOptions.blockTrackers,
            blockMalware = vmState.value.contentBlockersOptions.blockMalware,
            blockAdultContent = vmState.value.contentBlockersOptions.blockAdultContent,
            blockGambling = isEnabled,
        )
        onContentBlockerSettingsChanged(contentBlockerOptions)
    }

    fun onRemoveDnsClick() =
        viewModelScope.launch(dispatcher) {
            val dialog =
                vmState.value.dialogState as? AdvancedSettingsDialogState.DnsDialog ?: return@launch

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

    private fun onContentBlockerSettingsChanged(contentBlockersOption: DefaultDnsOptions) =
        viewModelScope.launch(dispatcher) {
            repository.setDnsOptions(
                isCustomDnsEnabled = vmState.value.isCustomDnsEnabled,
                dnsList = vmState.value.customDnsList.map { it.address }.asInetAddressList(),
                contentBlockersOptions = contentBlockersOption
            )
        }

    private fun hideDialog() {
        dialogState.update { AdvancedSettingsDialogState.NoDialog }
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

    private fun Settings.isCustomDnsEnabled() = tunnelOptions.dnsOptions.state == DnsState.Custom

    private fun Settings.addresses() = tunnelOptions.dnsOptions.customOptions.addresses

    private fun Settings.contentBlockersSettings() = tunnelOptions.dnsOptions.defaultOptions

    private fun String.isValidIp(): Boolean {
        return inetAddressValidator.isValid(this)
    }

    private fun String.isLocalAddress(): Boolean {
        return isValidIp() && InetAddress.getByName(this).isLocalAddress()
    }

    private fun InetAddress.isLocalAddress(): Boolean {
        return isLinkLocalAddress || isSiteLocalAddress
    }

    companion object {
        private const val EMPTY_STRING = ""
    }
}

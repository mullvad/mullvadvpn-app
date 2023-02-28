package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.isValidMtu
import org.apache.commons.validator.routines.InetAddressValidator

const val NO_EDIT_MODE = -1

sealed interface AdvancedSettingUiState {

    val mtu: String?
    val isCustomDnsEnabled: Boolean
    val customDnsList: List<String>
    val editDnsIndex: Int
    val currentEditValue: String

    fun isInEditMode(address: String?): Boolean {
        return if (address != null) {
            customDnsList.indexOf(address) == editDnsIndex
        } else {
            editDnsIndex == customDnsList.size
        }
    }

    data class NormalState(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val customDnsList: List<String>,
        override val editDnsIndex: Int = NO_EDIT_MODE,
        override val currentEditValue: String = ""
    ) : AdvancedSettingUiState

    data class EditMtu(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val customDnsList: List<String>,
        override val editDnsIndex: Int = NO_EDIT_MODE,
        override val currentEditValue: String = ""
    ) : AdvancedSettingUiState

    data class InsertLocalDns(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val customDnsList: List<String>,
        override val editDnsIndex: Int = NO_EDIT_MODE,
        override val currentEditValue: String = ""
    ) : AdvancedSettingUiState
}

enum class SettingScreenState {
    Normal,
    EditMtu,
    ConfirmLocalDns
}

private data class AdvancedSettingViewModelState(
    val mtuValue: String,
    val mode: SettingScreenState = SettingScreenState.Normal,
    val isCustomDnsEnabled: Boolean,
    val customDnsList: List<String>,
    val editDnsIndex: Int = NO_EDIT_MODE,
    val currentEditValue: String = "",
    val hasLocalChange: Boolean = false
) {
    fun toUiState(): AdvancedSettingUiState {
        return when (mode) {
            SettingScreenState.EditMtu -> AdvancedSettingUiState.EditMtu(
                mtu = mtuValue,
                isCustomDnsEnabled = isCustomDnsEnabled,
                customDnsList = customDnsList,
                editDnsIndex = editDnsIndex,
                currentEditValue = currentEditValue,
            )
            SettingScreenState.ConfirmLocalDns -> AdvancedSettingUiState.InsertLocalDns(
                mtu = mtuValue,
                isCustomDnsEnabled = isCustomDnsEnabled,
                customDnsList = customDnsList,
                editDnsIndex = editDnsIndex,
                currentEditValue = currentEditValue
            )
            else -> AdvancedSettingUiState.NormalState(
                mtu = mtuValue,
                isCustomDnsEnabled = isCustomDnsEnabled,
                customDnsList = customDnsList,
                editDnsIndex = editDnsIndex,
                currentEditValue = currentEditValue,
            )
        }
    }
}

class AdvancedSettingViewModel(
    private val repository: SettingsRepository,
    private val inetAddressValidator: InetAddressValidator,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {
    private val viewModelState = MutableStateFlow(
        AdvancedSettingViewModelState(
            mtuValue = repository.wireguardMtuString,
            mode = SettingScreenState.Normal,
            isCustomDnsEnabled = repository.customDns?.isCustomDnsEnabled() ?: false,
            customDnsList = repository.customDns?.onDnsServersChanged?.latestEvent?.map {
                it.hostAddress
            } as List<String>?
                ?: emptyList()
        )
    )

    val uiState = viewModelState
        .map(AdvancedSettingViewModelState::toUiState)
        .stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(),
            viewModelState.value.toUiState()
        )

    init {
        refreshSettings()
    }

    private fun refreshSettings() {
        viewModelState.update {
            it.copy(
                mtuValue = repository.wireguardMtuString,
                mode = SettingScreenState.Normal,
            )
        }

        viewModelScope.launch {
            val result = repository.fetchSettings()
            viewModelState.update {
                it.copy(
                    mtuValue = result.toString(),
                    mode = SettingScreenState.Normal,
                )
            }
        }
    }

    // Mtu manipulation functions
    fun onMtuChanged(newValue: String) {
        viewModelState.update {
            it.copy(mtuValue = newValue)
        }
    }

    fun onSubmitMtu() {
        saveMtu(viewModelState.value.mtuValue)
    }

    // This function handles the focus gain of MTU
    // for now it will clear Editing Dns index
    fun onMtuFocusChanged(hasFocus: Boolean) {
        if (hasFocus) {
            setEditDnsIndex(-1)
        }
    }

    // Dns manipulation functions
    fun toggleCustomDns(checked: Boolean) {
        viewModelScope.launch(dispatcher) {
            repository.setDnsOptions(
                checked,
                dnsList = viewModelState.value.customDnsList.map {
                    InetAddress.getByName(it)
                }
            )
            setEditDnsIndex(-1)
            viewModelState.update {
                it.copy(isCustomDnsEnabled = checked)
            }
        }
    }

    fun confirmDns(index: Int, addressText: String) {

        if (inetAddressValidator.isValid(addressText)) {
            val address = InetAddress.getByName(addressText)
            if (!address.isLoopbackAddress && !isAddressDuplicate(index, addressText)) {
                if (shouldShowLocalDnsWarningDialog(address)) {
                    viewModelState.update { vmUiState ->
                        vmUiState.copy(
                            mode = SettingScreenState.ConfirmLocalDns
                        )
                    }
                } else {
                    addDns(index, address)
                }
            }
        }
    }

    fun onConfirmAddLocalDns() {
        addDns(uiState.value.editDnsIndex, InetAddress.getByName(uiState.value.currentEditValue))
        viewModelState.update { vmUiState ->
            vmUiState.copy(
                mode = SettingScreenState.Normal
            )
        }
    }

    fun onCancelLocalDns() {
        viewModelState.update { vmUiState ->
            vmUiState.copy(
                mode = SettingScreenState.Normal
            )
        }
    }

    fun removeDnsClicked(index: Int) {
        setEditDnsIndex(-1)
        var list = viewModelState.value.customDnsList.toMutableList()
        viewModelState.update {
            list.removeAt(index)
            val shouldCustomDnsRemainEnable = list.size > 0 && it.isCustomDnsEnabled
            repository.setDnsOptions(
                isCustom = shouldCustomDnsRemainEnable,
                dnsList = list.map { item -> InetAddress.getByName(item) }
            )
            it.copy(
                isCustomDnsEnabled = shouldCustomDnsRemainEnable,
                customDnsList = list,
                hasLocalChange = true

            )
        }
    }

    fun dnsChanged(index: Int, addressText: String) {
        setEditDnsIndex(index)
        viewModelState.update {
            it.copy(
                editDnsIndex = index,
                currentEditValue = addressText
            )
        }
    }

//

    private fun addDns(index: Int, address: InetAddress) {
        viewModelState.value.let {
            var list = it.customDnsList.toMutableList()
            if (index == list.size) {
                list.add(address.hostAddress)
            } else if (index < list.size) {
                list[index] = address.hostAddress
            }
            repository.setDnsOptions(
                isCustom = it.isCustomDnsEnabled,
                dnsList = list.map { item -> InetAddress.getByName(item) }
            )
            setEditDnsIndex(-1)
            viewModelState.update { vmUiState ->
                vmUiState.copy(
                    customDnsList = list,
                    hasLocalChange = true
                )
            }
        }
    }

    private fun saveMtu(newValue: String) {
        var mtuValue = newValue.toIntOrNull()
        if (mtuValue?.isValidMtu() != false) {
            repository.wireguardMtu = mtuValue
        }
    }

    private fun shouldShowLocalDnsWarningDialog(address: InetAddress): Boolean {
        val isLocalAddress = address.isLinkLocalAddress || address.isSiteLocalAddress
        return isLocalAddress && !repository.isLocalNetworkSharingEnabled()
    }

    fun setEditDnsIndex(index: Int) {
        if (index != viewModelState.value.editDnsIndex) {
            var editValue = ""
            if (index in 0 until viewModelState.value.customDnsList.toMutableList().size) {
                editValue = viewModelState.value.customDnsList.toMutableList()[index]
            }

            viewModelState.update {
                it.copy(editDnsIndex = index, currentEditValue = editValue)
            }
        }
    }

    private fun isAddressDuplicate(index: Int, addressText: String): Boolean {
        return viewModelState.value.customDnsList.filterIndexed { i, item ->
            index != i && item == addressText
        }.isNotEmpty()
    }

    fun indexLostFocus(index: Int) {
//
//        if (index == viewModelState.value.editDnsIndex)
//            viewModelState.update {
//                it.copy(editDnsIndex = -1)
//            }
    }
}

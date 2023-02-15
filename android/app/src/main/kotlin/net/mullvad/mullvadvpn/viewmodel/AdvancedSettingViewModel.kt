package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.SettingsRepository
import org.apache.commons.validator.routines.InetAddressValidator

const val NO_EDIT_MODE = -1

sealed interface AdvancedSettingUiState {

    val mtu: String?
    val isCustomDnsEnabled: Boolean
    val customDnsList: List<String>
    val editDnsIndex: Int

    fun isInEditMode(address: String?): Boolean {
        return address?.let {
            customDnsList.indexOf(it) == editDnsIndex
        }
            ?: run {
                editDnsIndex == customDnsList.size
            }
    }

    data class NormalState(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val customDnsList: List<String>,
        override val editDnsIndex: Int = NO_EDIT_MODE
    ) : AdvancedSettingUiState

    data class EditMtu(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val customDnsList: List<String>,
        override val editDnsIndex: Int = NO_EDIT_MODE
    ) : AdvancedSettingUiState

    data class InsertLocalDns(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val customDnsList: List<String>,
        override val editDnsIndex: Int = NO_EDIT_MODE,
        val onConfirm: () -> Unit,
        val onCancel: () -> Unit,
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
    val hasLocalChange: Boolean = false
) {
    fun toUiState(): AdvancedSettingUiState {
        return when (mode) {
            SettingScreenState.EditMtu -> AdvancedSettingUiState.EditMtu(
                mtu = mtuValue,
                isCustomDnsEnabled = isCustomDnsEnabled,
                customDnsList = customDnsList,
                editDnsIndex = editDnsIndex
            )
            SettingScreenState.ConfirmLocalDns -> AdvancedSettingUiState.InsertLocalDns(
                mtu = mtuValue,
                isCustomDnsEnabled = isCustomDnsEnabled,
                customDnsList = customDnsList,
                editDnsIndex = editDnsIndex,
                {}, // add dns to repository
                {} // cancel confirm dialog
            )
            else -> AdvancedSettingUiState.NormalState(
                mtu = mtuValue,
                isCustomDnsEnabled = isCustomDnsEnabled,
                customDnsList = customDnsList,
                editDnsIndex = editDnsIndex
            )
        }
    }
}

/**
 * ViewModel that handles the business logic of the Setting screen
 */
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

    // UI state exposed to the UI
    val uiState = viewModelState
        .map(AdvancedSettingViewModelState::toUiState)
        .stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(),
            viewModelState.value.toUiState()
        )

    init {
        refreshSettings()

        // Observe for favorite changes in the repo layer
        viewModelScope.launch {
            repository.observeSettings().collectLatest { settings ->
                viewModelState.update {
                    it.copy(
                        mtuValue = settings.mtu,
                        mode = SettingScreenState.Normal,
                        isCustomDnsEnabled = settings.isCustomDnsEnabled
                    )
                }
            }
        }
    }

    /**
     * Refresh advanced settings and update the UI state accordingly
     */
    private fun refreshSettings() {
        // Ui state is refreshing
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
            repository.setCustomDnsEnabled(checked)
            viewModelState.update {
                it.copy(isCustomDnsEnabled = checked)
            }
        }
    }

    fun confirmDns(index: Int, addressText: String) {

        if (inetAddressValidator.isValid(addressText)) {
            val address = InetAddress.getByName(addressText)
            if (!address.isLoopbackAddress) {
                if (shouldShowLocalDnsWarningDialog(address)) {
                } else {
                    viewModelState.value.let {
                        var list = it.customDnsList.toMutableList()
                        if (index == list.size) {
                            list.add(address.hostAddress)
                        } else if (index < list.size) {
                            list[index] = address.hostAddress
                        }
                        setEditDnsIndex(-1)
                        viewModelState.update { vmUiState ->
                            vmUiState.copy(
                                customDnsList = list,
                                hasLocalChange = true
                            )
                        }
                    }
                }
            }
        }
    }

    fun removeDnsClicked(index: Int) {
        setEditDnsIndex(-1)
        var list = viewModelState.value.customDnsList.toMutableList()
        viewModelState.update {
            list.removeAt(index)
            it.copy(
                customDnsList = list,
                hasLocalChange = true

            )
//            repository.removeDns(InetAddress.getByAddress(list[index]))
        }
    }

    fun dnsChanged(index: Int, addressText: String) {
        var list = viewModelState.value.customDnsList.toMutableList()
        list[index] = addressText
        viewModelState.update {
            it.copy(
                customDnsList = list,
            )
        }
    }

//

    private fun saveMtu(newValue: String) {
        if (isValidMtu(newValue)) {
            repository.wireguardMtu = newValue.toIntOrNull()
        }
    }

    private fun isValidMtu(newValue: String): Boolean {
        return newValue.toIntOrNull()?.let {
            it in 1280..1420
        } ?: run { true }
    }

    private fun shouldShowLocalDnsWarningDialog(address: InetAddress): Boolean {
        val isLocalAddress = address.isLinkLocalAddress || address.isSiteLocalAddress
        return isLocalAddress && !repository.isLocalNetworkSharingEnabled()
    }

    fun setEditDnsIndex(index: Int) {
        viewModelState.update {
            it.copy(editDnsIndex = index)
        }
    }

    fun indexLostFocus(index: Int) {
//
//        if (index == viewModelState.value.editDnsIndex)
//            viewModelState.update {
//                it.copy(editDnsIndex = -1)
//            }
    }
}

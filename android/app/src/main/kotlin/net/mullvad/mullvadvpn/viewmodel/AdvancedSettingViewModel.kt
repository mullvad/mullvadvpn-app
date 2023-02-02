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
import org.apache.commons.validator.routines.InetAddressValidator

const val NO_EDIT_MODE = -1

sealed interface AdvancedSettingUiState {

    val mtu: String?
    val isCustomDnsEnabled: Boolean
    val customDnsList: List<InetAddress>
    val editDnsIndex: Int

    fun isInEditMode(address: InetAddress?): Boolean {
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
        override val customDnsList: List<InetAddress>,
        override val editDnsIndex: Int = NO_EDIT_MODE
    ) : AdvancedSettingUiState

    data class EditMtu(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val customDnsList: List<InetAddress>,
        override val editDnsIndex: Int = NO_EDIT_MODE
    ) : AdvancedSettingUiState

    data class InsertLocalDns(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val customDnsList: List<InetAddress>,
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
    val customDnsList: List<InetAddress>,
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
            customDnsList = repository.customDns?.onDnsServersChanged?.latestEvent ?: emptyList()
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
            repository.observeSettings().collect { settings ->
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

    fun toggleCustomDns(checked: Boolean) {
        viewModelScope.launch(dispatcher) {
            repository.setCustomDnsEnabled(checked)
            viewModelState.update {
                it.copy(isCustomDnsEnabled = checked)
            }
        }
    }

    fun addDnsClicked(addressText: String) {
        viewModelScope.launch(dispatcher) {
            if (inetAddressValidator.isValid(addressText)) {
                val address = InetAddress.getByName(addressText)
                if (!address.isLoopbackAddress) {
                    if (shouldShowLocalDnsWarningDialog(address)) {
                    } else {
                        viewModelState.value.let {
                            it.copy(
                                customDnsList = it.customDnsList.toMutableList()
                                    .apply {
                                        add(address)
                                    },
                                hasLocalChange = true
                            )
                        }
                    }
                }
            }
        }
    }

    fun editDnsClicked(index: Int, addressText: String) {
        viewModelScope.launch(dispatcher) {
            if (inetAddressValidator.isValid(addressText)) {
                val address = InetAddress.getByName(addressText)
                if (!address.isLoopbackAddress()) {
                    if (shouldShowLocalDnsWarningDialog(address)) {
                    } else {
                        viewModelState.value.let {
                            var list = it.customDnsList.toMutableList()
                            list.set(index, address)
                            it.copy(
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
        viewModelScope.launch(dispatcher) {
            viewModelState.value.let {
                var list = it.customDnsList.toMutableList()
                list.removeAt(index)
                it.copy(
                    customDnsList = list,
                    hasLocalChange = true
                )
            }
        }
    }

    fun dnsChanged(index: Int, addressText: String) {
        viewModelScope.launch(dispatcher) {
            if (inetAddressValidator.isValid(addressText)) {
                val address = InetAddress.getByName(addressText)
                if (!address.isLoopbackAddress()) {
                    if (shouldShowLocalDnsWarningDialog(address)) {
                    } else {
                        viewModelState.value.let {
                            it.copy(
                                customDnsList = it.customDnsList.toMutableList()
                                    .apply {
                                        add(address)
                                    },
                                hasLocalChange = true
                            )
                        }
                    }
                }
            }
        }
    }

    fun onMtuChanged(newValue: String) {
        viewModelState.update {
            it.copy(mtuValue = newValue)
        }
    }

    fun onSubmitMtu() {
        saveMtu(viewModelState.value.mtuValue)
    }

    fun clearEnteredDns() {
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
        val isLocalAddress = address.isLinkLocalAddress() || address.isSiteLocalAddress()
        return isLocalAddress || !repository.isLocalNetworkSharingEnabled()
    }

    fun setEditDnsIndex(index: Int) {
        viewModelState.update {
            it.copy(editDnsIndex = index)
        }
    }
}

package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
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

sealed interface AdvancedSettingUiState {
    val mtu: String?
    val isCustomDnsEnabled: Boolean

    data class NormalState(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
    ) : AdvancedSettingUiState

    data class EditMtu(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
    ) : AdvancedSettingUiState
}

private data class AdvancedSettingViewModelState(
    val mtuValue: String,
    val isMtuEditMode: Boolean = false,
    val isCustomDnsEnabled: Boolean,
    val customDnsList: List<String> = emptyList()
) {
    fun toUiState(): AdvancedSettingUiState {
        return if (isMtuEditMode) {
            AdvancedSettingUiState.EditMtu(
                mtu = mtuValue,
                isCustomDnsEnabled = isCustomDnsEnabled
            )
        } else {
            AdvancedSettingUiState.NormalState(
                mtu = mtuValue,
                isCustomDnsEnabled = isCustomDnsEnabled
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
            isMtuEditMode = false,
            isCustomDnsEnabled = repository.customDns?.isCustomDnsEnabled() ?: false
        )
    )

    // UI state exposed to the UI
    val uiState = viewModelState
        .map(AdvancedSettingViewModelState::toUiState)
        .stateIn(
            viewModelScope,
            SharingStarted.Eagerly,
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
                        isMtuEditMode = false,
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
                isMtuEditMode = false,
            )
        }

        viewModelScope.launch {
            val result = repository.fetchSettings()
            viewModelState.update {
                it.copy(
                    mtuValue = result.toString(),
                    isMtuEditMode = false,
                )
            }
        }
    }

    fun addDns(dns: String) {
        viewModelScope.launch(dispatcher) {
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

    fun toggleCustomDns(checked: Boolean) {
        viewModelScope.launch(dispatcher) {
            repository.setCustomDnsEnabled(checked)
            viewModelState.update {
                it.copy(isCustomDnsEnabled = checked)
            }
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
}

sealed class DnsSettingsState {
    class Normal(var dns: String) : DnsSettingsState()
    class AddLocalDnsConfirm(var localDns: String) : DnsSettingsState()

    fun newDns(): String? {
        return when (this) {
            is Normal -> (this as? Normal)?.dns
            is AddLocalDnsConfirm -> (this as? AddLocalDnsConfirm)?.localDns
            else -> null
        }
    }
}

sealed class CellUiState {
    var showWarning: Boolean = false

    class MTUCellUiState() : CellUiState()
    class CustomDNSCellUiState() : CellUiState()
    object Hide : CellUiState()
}

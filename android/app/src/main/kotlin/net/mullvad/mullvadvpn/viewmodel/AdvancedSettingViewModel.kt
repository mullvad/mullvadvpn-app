package net.mullvad.mullvadvpn.viewmodel

import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.ViewModel
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.ui.fragments.SplitTunnelingFragment
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import org.apache.commons.validator.routines.InetAddressValidator
import java.net.InetAddress

sealed interface AdvancedSettingUiState {

    val mtu: String?
    val isCustomDnsEnabled: Boolean
    val customDnsList: List<InetAddress>

    data class NormalState(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val customDnsList: List<InetAddress>
    ) : AdvancedSettingUiState

    data class EditMtu(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val customDnsList: List<InetAddress>
    ) : AdvancedSettingUiState

    data class InsertLocalDns(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val customDnsList: List<InetAddress>,
        val onConfirm: () -> Unit,
        val onCancel: () -> Unit,
    ) : AdvancedSettingUiState
}

enum class SettingScreenState {
    Normal,
    EditMtu,
    EditDns,
    NewDns,
    ConfirmLocalDns
}

private data class AdvancedSettingViewModelState(
    val mtuValue: String,
    val mode: SettingScreenState = SettingScreenState.Normal,
    val isCustomDnsEnabled: Boolean,
    val customDnsList: List<InetAddress>
) {
    fun toUiState(): AdvancedSettingUiState {
        return when (mode) {
            SettingScreenState.EditMtu -> AdvancedSettingUiState.EditMtu(
                mtu = mtuValue,
                isCustomDnsEnabled = isCustomDnsEnabled,
                customDnsList = customDnsList
            )
            SettingScreenState.EditDns -> AdvancedSettingUiState.NormalState(
                mtu = mtuValue,
                isCustomDnsEnabled = isCustomDnsEnabled,
                customDnsList = customDnsList
            )
            SettingScreenState.NewDns -> AdvancedSettingUiState.NormalState(
                mtu = mtuValue,
                isCustomDnsEnabled = isCustomDnsEnabled,
                customDnsList = customDnsList
            )
            SettingScreenState.ConfirmLocalDns -> AdvancedSettingUiState.InsertLocalDns(
                mtu = mtuValue,
                isCustomDnsEnabled = isCustomDnsEnabled,
                customDnsList = customDnsList,
                {},
                {}
            )
            else ->
                AdvancedSettingUiState.NormalState(
                    mtu = mtuValue,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    customDnsList = customDnsList
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

    fun addDnsClicked(addressText: String) {
        viewModelScope.launch(dispatcher) {
            if (inetAddressValidator.isValid(addressText)) {
                val address = InetAddress.getByName(addressText)
                if (!address.isLoopbackAddress()) {
                    if (shouldShowLocalDnsWarningDialog(address)) {

                    } else {
                        repository.addDns(address)
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

    private fun shouldShowLocalDnsWarningDialog(address: InetAddress): Boolean {
        val isLocalAddress = address.isLinkLocalAddress() || address.isSiteLocalAddress()
        return isLocalAddress || !repository.isLocalNetworkSharingEnabled()
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

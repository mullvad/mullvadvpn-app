package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.constant.EMPTY_STRING
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.repository.SettingsRepository
import org.apache.commons.validator.routines.InetAddressValidator

sealed interface DnsDialogSideEffect {
    data object Complete : DnsDialogSideEffect
}

data class DnsDialogViewModelState(
    val customDnsList: List<InetAddress>,
    val isAllowLanEnabled: Boolean
) {
    companion object {
        fun default() = DnsDialogViewModelState(emptyList(), false)
    }
}

data class DnsDialogViewState(
    val ipAddress: String,
    val validationResult: ValidationResult = ValidationResult.Success,
    val isLocal: Boolean,
    val isAllowLanEnabled: Boolean,
    val isNewEntry: Boolean
) {

    fun isValid() = (validationResult is ValidationResult.Success)

    sealed class ValidationResult {
        data object Success : ValidationResult()

        data object InvalidAddress : ValidationResult()

        data object DuplicateAddress : ValidationResult()
    }
}

class DnsDialogViewModel(
    private val repository: SettingsRepository,
    private val inetAddressValidator: InetAddressValidator,
    private val index: Int? = null,
    initialValue: String?,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {

    private val _ipAddressInput = MutableStateFlow(initialValue ?: EMPTY_STRING)

    private val vmState =
        repository.settingsUpdates
            .filterNotNull()
            .map {
                val customDnsList = it.addresses()
                val isAllowLanEnabled = it.allowLan
                DnsDialogViewModelState(customDnsList, isAllowLanEnabled = isAllowLanEnabled)
            }
            .stateIn(viewModelScope, SharingStarted.Lazily, DnsDialogViewModelState.default())

    val uiState: StateFlow<DnsDialogViewState> =
        combine(_ipAddressInput, vmState, ::createViewState)
            .stateIn(
                viewModelScope,
                SharingStarted.Lazily,
                createViewState(_ipAddressInput.value, vmState.value)
            )

    private val _uiSideEffect = Channel<DnsDialogSideEffect>(1, BufferOverflow.DROP_OLDEST)
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun createViewState(ipAddress: String, vmState: DnsDialogViewModelState) =
        DnsDialogViewState(
            ipAddress,
            ipAddress.validateDnsEntry(index, vmState.customDnsList),
            ipAddress.isLocalAddress(),
            isAllowLanEnabled = vmState.isAllowLanEnabled,
            index == null
        )

    private fun String.validateDnsEntry(
        index: Int?,
        dnsList: List<InetAddress>
    ): DnsDialogViewState.ValidationResult =
        when {
            this.isBlank() || !this.isValidIp() -> {
                DnsDialogViewState.ValidationResult.InvalidAddress
            }
            InetAddress.getByName(this).isDuplicateDnsEntry(index, dnsList) -> {
                DnsDialogViewState.ValidationResult.DuplicateAddress
            }
            else -> DnsDialogViewState.ValidationResult.Success
        }

    fun onDnsInputChange(ipAddress: String) {
        _ipAddressInput.value = ipAddress
    }

    fun onSaveDnsClick() =
        viewModelScope.launch(dispatcher) {
            if (!uiState.value.isValid()) return@launch

            val address = InetAddress.getByName(uiState.value.ipAddress)

            repository.updateCustomDnsList {
                it.toMutableList().apply {
                    if (index != null) {
                        set(index, address)
                    } else {
                        add(address)
                    }
                }
            }

            _uiSideEffect.send(DnsDialogSideEffect.Complete)
        }

    fun onRemoveDnsClick() =
        viewModelScope.launch(dispatcher) {
            repository.updateCustomDnsList {
                it.filter { it.hostAddress != uiState.value.ipAddress }
            }
            _uiSideEffect.send(DnsDialogSideEffect.Complete)
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

    private fun InetAddress.isDuplicateDnsEntry(
        currentIndex: Int? = null,
        dnsList: List<InetAddress>
    ): Boolean =
        dnsList.withIndex().any { (index, entry) ->
            if (index == currentIndex) {
                // Ignore current index, it may be the same
                false
            } else {
                entry == this
            }
        }

    private fun Settings.addresses() = tunnelOptions.dnsOptions.customOptions.addresses

    companion object {
        private const val EMPTY_STRING = ""
    }
}

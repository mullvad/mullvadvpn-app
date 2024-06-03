package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.SettingsRepository
import org.apache.commons.validator.routines.InetAddressValidator

sealed interface DnsDialogSideEffect {
    data object Complete : DnsDialogSideEffect

    data object Error : DnsDialogSideEffect
}

data class DnsDialogViewState(
    val input: String,
    val validationError: ValidationError?,
    val isLocal: Boolean,
    val isAllowLanEnabled: Boolean,
    val index: Int?,
) {
    val isNewEntry = index == null

    fun isValid() = validationError == null
}

sealed class ValidationError {
    data object InvalidAddress : ValidationError()

    data object DuplicateAddress : ValidationError()
}

class DnsDialogViewModel(
    private val repository: SettingsRepository,
    private val inetAddressValidator: InetAddressValidator,
    index: Int? = null,
    initialValue: String?,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {

    private val currentIndex = MutableStateFlow(index)
    private val _ipAddressInput = MutableStateFlow(initialValue ?: EMPTY_STRING)

    val uiState: StateFlow<DnsDialogViewState> =
        combine(_ipAddressInput, currentIndex, repository.settingsUpdates.filterNotNull()) {
                input,
                currentIndex,
                settings ->
                createViewState(settings.addresses(), currentIndex, settings.allowLan, input)
            }
            .stateIn(
                viewModelScope,
                SharingStarted.Lazily,
                createViewState(emptyList(), null, false, _ipAddressInput.value)
            )

    private val _uiSideEffect = Channel<DnsDialogSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun createViewState(
        customDnsList: List<InetAddress>,
        currentIndex: Int?,
        isAllowLanEnabled: Boolean,
        input: String
    ): DnsDialogViewState =
        DnsDialogViewState(
            input,
            input.validateDnsEntry(currentIndex, customDnsList).leftOrNull(),
            input.isLocalAddress(),
            isAllowLanEnabled = isAllowLanEnabled,
            currentIndex
        )

    private fun String.validateDnsEntry(
        index: Int?,
        dnsList: List<InetAddress>
    ): Either<ValidationError, InetAddress> = either {
        ensure(isNotBlank()) { ValidationError.InvalidAddress }
        ensure(isValidIp()) { ValidationError.InvalidAddress }
        val inetAddress = InetAddress.getByName(this@validateDnsEntry)
        ensure(!inetAddress.isDuplicateDnsEntry(index, dnsList)) {
            ValidationError.DuplicateAddress
        }
        inetAddress
    }

    fun onDnsInputChange(ipAddress: String) {
        _ipAddressInput.value = ipAddress
    }

    fun onSaveDnsClick() =
        viewModelScope.launch(dispatcher) {
            if (!uiState.value.isValid()) return@launch

            val address = InetAddress.getByName(uiState.value.input)

            val index = uiState.value.index
            val result =
                if (index != null) {
                    repository.setCustomDns(index = index, address = address)
                } else {
                    repository.addCustomDns(address = address).onRight { currentIndex.value = it }
                }

            result.fold(
                { _uiSideEffect.send(DnsDialogSideEffect.Error) },
                { _uiSideEffect.send(DnsDialogSideEffect.Complete) }
            )
        }

    fun onRemoveDnsClick(index: Int) =
        viewModelScope.launch(dispatcher) {
            repository
                .deleteCustomDns(index)
                .fold(
                    { _uiSideEffect.send(DnsDialogSideEffect.Error) },
                    { _uiSideEffect.send(DnsDialogSideEffect.Complete) }
                )
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

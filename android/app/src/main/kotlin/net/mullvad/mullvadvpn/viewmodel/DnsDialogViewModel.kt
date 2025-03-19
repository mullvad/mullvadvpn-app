package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import arrow.core.raise.either
import arrow.core.raise.ensure
import com.ramcosta.composedestinations.generated.destinations.DnsDestination
import java.net.Inet6Address
import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.usecase.DeleteCustomDnsUseCase
import org.apache.commons.validator.routines.InetAddressValidator

sealed interface DnsDialogSideEffect {
    data class Complete(val isDnsListEmpty: Boolean) : DnsDialogSideEffect

    data object Error : DnsDialogSideEffect
}

data class DnsDialogViewState(
    val input: String,
    val validationError: ValidationError?,
    val isLocal: Boolean,
    val isAllowLanEnabled: Boolean,
    val isIpv6: Boolean,
    val isIpv6Enabled: Boolean,
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
    savedStateHandle: SavedStateHandle,
    private val deleteCustomDnsUseCase: DeleteCustomDnsUseCase,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {
    private val navArgs = DnsDestination.argsFrom(savedStateHandle)

    private val settings = MutableStateFlow<Settings?>(null)
    private val currentIndex = MutableStateFlow(navArgs.index)
    private val _ipAddressInput = MutableStateFlow(navArgs.initialValue ?: EMPTY_STRING)

    val uiState: StateFlow<DnsDialogViewState> =
        combine(_ipAddressInput, currentIndex, settings.filterNotNull()) {
                input,
                currentIndex,
                settings ->
                DnsDialogViewState(
                    input = input,
                    validationError =
                        input.validateDnsEntry(currentIndex, settings.addresses()).leftOrNull(),
                    isLocal = input.isLocalAddress(),
                    isIpv6 = input.isIpv6(),
                    isAllowLanEnabled = settings.allowLan,
                    isIpv6Enabled = settings.tunnelOptions.genericOptions.enableIpv6,
                    index = currentIndex,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.Lazily,
                DnsDialogViewState(
                    input = _ipAddressInput.value,
                    validationError = null,
                    isLocal = _ipAddressInput.value.isLocalAddress(),
                    isIpv6 = _ipAddressInput.value.isIpv6(),
                    isAllowLanEnabled = false,
                    isIpv6Enabled = false,
                    index = null,
                ),
            )

    private val _uiSideEffect = Channel<DnsDialogSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    init {
        viewModelScope.launch { settings.emit(repository.settingsUpdates.filterNotNull().first()) }
    }

    private fun String.validateDnsEntry(
        index: Int?,
        dnsList: List<InetAddress>,
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
                    repository.addCustomDns(address = address)
                }

            result.fold(
                { _uiSideEffect.send(DnsDialogSideEffect.Error) },
                { _uiSideEffect.send(DnsDialogSideEffect.Complete(false)) },
            )
        }

    fun onRemoveDnsClick(index: Int) =
        viewModelScope.launch(dispatcher) {
            deleteCustomDnsUseCase
                .invoke(index)
                .fold(
                    { _uiSideEffect.send(DnsDialogSideEffect.Error) },
                    { _uiSideEffect.send(DnsDialogSideEffect.Complete(it == 0)) },
                )
        }

    private fun String.isValidIp(): Boolean {
        return inetAddressValidator.isValid(this)
    }

    private fun String.isLocalAddress(): Boolean {
        return isValidIp() && InetAddress.getByName(this).isLocalAddress()
    }

    private fun String.isIpv6(): Boolean {
        return isValidIp() && InetAddress.getByName(this) is Inet6Address
    }

    private fun InetAddress.isLocalAddress(): Boolean {
        return isLinkLocalAddress || isSiteLocalAddress
    }

    private fun InetAddress.isDuplicateDnsEntry(
        currentIndex: Int? = null,
        dnsList: List<InetAddress>,
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

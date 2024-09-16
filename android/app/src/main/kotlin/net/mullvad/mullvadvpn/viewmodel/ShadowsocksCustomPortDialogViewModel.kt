package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.ShadowsocksCustomPortDestination
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.util.inAnyOf

class ShadowsocksCustomPortDialogViewModel(
    savedStateHandle: SavedStateHandle,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {
    private val navArgs = ShadowsocksCustomPortDestination.argsFrom(savedStateHandle).navArg

    private val _portInput = MutableStateFlow(navArgs.customPort?.value?.toString() ?: "")
    private val _isValidPort = MutableStateFlow(_portInput.value.isValidPort())

    val uiState: StateFlow<ShadowsocksCustomPortDialogUiState> =
        combine(_portInput, _isValidPort, ::createState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                createState(_portInput.value, _isValidPort.value),
            )

    private val _uiSideEffect = Channel<ShadowsocksCustomPortDialogSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun createState(portInput: String, isValidPortInput: Boolean) =
        ShadowsocksCustomPortDialogUiState(
            portInput = portInput,
            isValidInput = isValidPortInput,
            allowedPortRanges = navArgs.allowedPortRanges,
            showResetToDefault = navArgs.customPort != null,
        )

    fun onInputChanged(value: String) {
        _portInput.value = value
        _isValidPort.value = value.isValidPort()
    }

    fun onSaveClick(portValue: String) =
        viewModelScope.launch(dispatcher) {
            val port = portValue.parseValidPort() ?: return@launch
            _uiSideEffect.send(ShadowsocksCustomPortDialogSideEffect.Success(port))
        }

    fun onResetClick() {
        viewModelScope.launch(dispatcher) {
            _uiSideEffect.send(ShadowsocksCustomPortDialogSideEffect.Success(null))
        }
    }

    private fun String.isValidPort(): Boolean = parseValidPort() != null

    private fun String.parseValidPort(): Port? =
        Port.fromString(this).getOrNull()?.takeIf { port ->
            port.inAnyOf(navArgs.allowedPortRanges)
        }
}

sealed interface ShadowsocksCustomPortDialogSideEffect {
    data class Success(val port: Port?) : ShadowsocksCustomPortDialogSideEffect
}

data class ShadowsocksCustomPortDialogUiState(
    val portInput: String,
    val isValidInput: Boolean,
    val allowedPortRanges: List<PortRange>,
    val showResetToDefault: Boolean,
)

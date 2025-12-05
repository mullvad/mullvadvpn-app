package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.CustomPortDestination
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.util.inAnyOf

class CustomPortDialogViewModel(
    savedStateHandle: SavedStateHandle,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {
    private val navArgs = CustomPortDestination.argsFrom(savedStateHandle).navArg

    private val _portInput = MutableStateFlow(navArgs.customPort?.value?.toString() ?: "")
    private val _isValidPort = MutableStateFlow(_portInput.value.isValidPort())

    val uiState: StateFlow<CustomPortDialogUiState> =
        combine(_portInput, _isValidPort, ::createState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                createState(_portInput.value, _isValidPort.value),
            )

    private val _uiSideEffect = Channel<CustomPortDialogSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun createState(portInput: String, isValidPortInput: Boolean) =
        CustomPortDialogUiState(
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
            _uiSideEffect.send(CustomPortDialogSideEffect.Success(port))
        }

    fun onResetClick() {
        viewModelScope.launch(dispatcher) {
            _uiSideEffect.send(CustomPortDialogSideEffect.Success(null))
        }
    }

    private fun String.isValidPort(): Boolean = parseValidPort() != null

    private fun String.parseValidPort(): Port? =
        Port.fromString(this).getOrNull()?.takeIf { port ->
            port.inAnyOf(navArgs.allowedPortRanges)
        }
}

sealed interface CustomPortDialogSideEffect {
    data class Success(val port: Port?) : CustomPortDialogSideEffect
}

data class CustomPortDialogUiState(
    val portInput: String,
    val isValidInput: Boolean,
    val allowedPortRanges: List<PortRange>,
    val showResetToDefault: Boolean,
)

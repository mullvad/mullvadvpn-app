package net.mullvad.mullvadvpn.feature.anticensorship.impl.customport

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import arrow.core.raise.context.bind
import arrow.core.raise.either
import arrow.core.raise.ensure
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
import net.mullvad.mullvadvpn.feature.anticensorship.api.CustomPortNavKey
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.util.inAnyOf
import net.mullvad.mullvadvpn.lib.model.ParsePortError
import net.mullvad.mullvadvpn.lib.model.Port

class CustomPortDialogViewModel(
    private val navArgs: CustomPortNavKey,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {

    private val _portInput = MutableStateFlow(navArgs.customPort?.value?.toString() ?: "")
    private val _portInputError = MutableStateFlow<ParsePortError?>(null)

    val uiState: StateFlow<CustomPortDialogUiState> =
        combine(_portInput, _portInputError, ::createState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                createState(_portInput.value, null),
            )

    private val _uiSideEffect = Channel<CustomPortDialogSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun createState(portInput: String, portInputError: ParsePortError?) =
        CustomPortDialogUiState(
            portInput = portInput,
            portInputError = portInputError,
            allowedPortRanges = navArgs.allowedPortRanges,
            recommendedPortRanges = navArgs.recommendedPortRanges,
            showResetToDefault = navArgs.customPort != null,
        )

    fun onInputChanged(value: String) {
        _portInput.value = value
        _portInputError.value = null
    }

    fun onSaveClick(portValue: String) =
        viewModelScope.launch(dispatcher) {
            portValue
                .parseValidPort()
                .fold(
                    { error -> _portInputError.value = error },
                    { port -> _uiSideEffect.send(CustomPortDialogSideEffect.Success(port)) },
                )
        }

    fun onResetClick() {
        viewModelScope.launch(dispatcher) {
            _uiSideEffect.send(CustomPortDialogSideEffect.Success(null))
        }
    }

    private fun String.parseValidPort(): Either<ParsePortError, Port> = either {
        val port = Port.fromString(this@parseValidPort).bind()
        ensure(port.inAnyOf(navArgs.allowedPortRanges)) { ParsePortError.OutOfRange(port.value) }
        port
    }
}

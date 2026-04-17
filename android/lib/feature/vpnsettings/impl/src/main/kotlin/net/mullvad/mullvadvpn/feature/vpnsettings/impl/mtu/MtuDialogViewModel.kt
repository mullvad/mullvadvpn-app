package net.mullvad.mullvadvpn.feature.vpnsettings.impl.mtu

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
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
import net.mullvad.mullvadvpn.feature.vpnsettings.api.MtuNavKey
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ParseMtuError
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository

class MtuDialogViewModel(
    private val navArgs: MtuNavKey,
    private val repository: SettingsRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {

    private val _mtuInput = MutableStateFlow(navArgs.initialMtu?.value?.toString() ?: "")
    private val _inputError = MutableStateFlow<ParseMtuError?>(null)

    val uiState: StateFlow<MtuDialogUiState> =
        combine(_mtuInput, _inputError, ::createState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                createState(_mtuInput.value, _inputError.value),
            )

    private val _uiSideEffect = Channel<MtuDialogSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun createState(mtuInput: String, parseMtuError: ParseMtuError?) =
        MtuDialogUiState(
            mtuInput = mtuInput,
            inputError = parseMtuError,
            showResetToDefault = navArgs.initialMtu != null,
        )

    fun onInputChanged(value: String) {
        _mtuInput.value = value
        _inputError.value = null
    }

    fun onSaveClick(mtuValue: String) =
        viewModelScope.launch(dispatcher) {
            val mtu =
                Mtu.fromString(mtuValue)
                    .fold(
                        {
                            _inputError.value = it
                            return@launch
                        },
                        { it },
                    )
            repository
                .setWireguardMtu(mtu)
                .fold(
                    { _uiSideEffect.send(MtuDialogSideEffect.Error) },
                    { _uiSideEffect.send(MtuDialogSideEffect.Complete) },
                )
        }

    fun onRestoreClick() =
        viewModelScope.launch(dispatcher) {
            repository
                .resetWireguardMtu()
                .fold(
                    { _uiSideEffect.send(MtuDialogSideEffect.Error) },
                    { _uiSideEffect.send(MtuDialogSideEffect.Complete) },
                )
        }
}

sealed interface MtuDialogSideEffect {
    data object Complete : MtuDialogSideEffect

    data object Error : MtuDialogSideEffect
}

data class MtuDialogUiState(
    val mtuInput: String,
    val inputError: ParseMtuError?,
    val showResetToDefault: Boolean,
)

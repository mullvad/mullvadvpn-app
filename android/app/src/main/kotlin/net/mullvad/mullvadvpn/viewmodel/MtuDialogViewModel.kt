package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.MtuDestination
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
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.repository.SettingsRepository

class MtuDialogViewModel(
    private val repository: SettingsRepository,
    savedStateHandle: SavedStateHandle,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {
    private val navArgs = MtuDestination.argsFrom(savedStateHandle)

    private val _mtuInput = MutableStateFlow(navArgs.initialMtu?.value?.toString() ?: "")
    private val _isValidMtu = MutableStateFlow(true)

    val uiState: StateFlow<MtuDialogUiState> =
        combine(_mtuInput, _isValidMtu, ::createState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                createState(_mtuInput.value, _isValidMtu.value)
            )

    private val _uiSideEffect = Channel<MtuDialogSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private fun createState(mtuInput: String, isValidMtuInput: Boolean) =
        MtuDialogUiState(
            mtuInput = mtuInput,
            isValidInput = isValidMtuInput,
            showResetToDefault = navArgs.initialMtu != null
        )

    fun onInputChanged(value: String) {
        _mtuInput.value = value
        _isValidMtu.value = Mtu.fromString(value).isRight()
    }

    fun onSaveClick(mtuValue: String) =
        viewModelScope.launch(dispatcher) {
            val mtu = Mtu.fromString(mtuValue).getOrNull() ?: return@launch
            repository
                .setWireguardMtu(mtu)
                .fold(
                    { _uiSideEffect.send(MtuDialogSideEffect.Error) },
                    { _uiSideEffect.send(MtuDialogSideEffect.Complete) }
                )
        }

    fun onRestoreClick() =
        viewModelScope.launch(dispatcher) {
            repository
                .resetWireguardMtu()
                .fold(
                    { _uiSideEffect.send(MtuDialogSideEffect.Error) },
                    { _uiSideEffect.send(MtuDialogSideEffect.Complete) }
                )
        }
}

sealed interface MtuDialogSideEffect {
    data object Complete : MtuDialogSideEffect

    data object Error : MtuDialogSideEffect
}

data class MtuDialogUiState(
    val mtuInput: String,
    val isValidInput: Boolean,
    val showResetToDefault: Boolean
)

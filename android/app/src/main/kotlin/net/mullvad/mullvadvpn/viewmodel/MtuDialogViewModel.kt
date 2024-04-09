package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.isValidMtu

class MtuDialogViewModel(
    private val repository: SettingsRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {

    private val _uiSideEffect = Channel<MtuDialogSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun onSaveClick(mtuValue: Int) =
        viewModelScope.launch(dispatcher) {
            if (mtuValue.isValidMtu()) {
                repository
                    .setWireguardMtu(mtuValue)
                    .fold(
                        { _uiSideEffect.send(MtuDialogSideEffect.Error) },
                        { _uiSideEffect.send(MtuDialogSideEffect.Complete) }
                    )
            }
        }

    fun onRestoreClick() =
        viewModelScope.launch(dispatcher) {
            repository
                .setWireguardMtu(null)
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

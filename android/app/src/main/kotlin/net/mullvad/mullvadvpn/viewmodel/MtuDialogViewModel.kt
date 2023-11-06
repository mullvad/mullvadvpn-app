package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.isValidMtu

class MtuDialogViewModel(
    private val repository: SettingsRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {

    private val _uiSideEffect = MutableSharedFlow<MtuDialogSideEffect>()
    val uiSideEffect: SharedFlow<MtuDialogSideEffect> = _uiSideEffect

    fun onSaveClick(mtuValue: Int) =
        viewModelScope.launch(dispatcher) {
            if (mtuValue.isValidMtu()) {
                repository.setWireguardMtu(mtuValue)
            }
            _uiSideEffect.emit(MtuDialogSideEffect.Complete)
        }

    fun onRestoreClick() =
        viewModelScope.launch(dispatcher) {
            repository.setWireguardMtu(null)
            _uiSideEffect.emit(MtuDialogSideEffect.Complete)
        }
}

sealed interface MtuDialogSideEffect {
    data object Complete : MtuDialogSideEffect
}

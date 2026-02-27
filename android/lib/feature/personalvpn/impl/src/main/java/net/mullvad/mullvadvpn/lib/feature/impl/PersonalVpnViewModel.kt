package net.mullvad.mullvadvpn.lib.feature.impl

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.toLc

class PersonalVpnViewModel(savedStateHandle: SavedStateHandle) : ViewModel() {
    private val _uiSideEffect = Channel<PersonalVpnSideEffect>()
    val uiSideEffect = merge(_uiSideEffect.receiveAsFlow())
    val uiState: StateFlow<Lc<Boolean, PersonalVpnUiState>> =
        flowOf<Lc<Boolean, PersonalVpnUiState>>(PersonalVpnUiState(true).toLc())
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(false),
            )

    fun onToggle(on: Boolean): Unit = TODO()
}

sealed interface PersonalVpnSideEffect {}

data class PersonalVpnUiState(
    val enabled: Boolean
)

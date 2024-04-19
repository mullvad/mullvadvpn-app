package net.mullvad.mullvadvpn.viewmodel

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import net.mullvad.mullvadvpn.lib.common.util.MigrateSplitTunnelingResult
import net.mullvad.mullvadvpn.repository.SplitTunnelingRepository

class SplitTunnelingViewModel(
    private val appsProvider: ApplicationsProvider,
    private val splitTunnelingRepository: SplitTunnelingRepository,
    private val dispatcher: CoroutineDispatcher
) : ViewModel() {

    private val _uiSideEffect = Channel<SplitTunnelingUiSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private val allApps = MutableStateFlow<List<AppData>?>(null)
    private val showSystemApps = MutableStateFlow(false)

    private val vmState: StateFlow<SplitTunnelingViewModelState> =
        combine(
                splitTunnelingRepository.excludedApps,
                splitTunnelingRepository.splitTunnelingEnabled,
                allApps,
                showSystemApps,
            ) { excludedApps, enabled, allApps, showSystemApps ->
                SplitTunnelingViewModelState(
                    excludedApps = excludedApps,
                    enabled = enabled,
                    allApps = allApps,
                    showSystemApps = showSystemApps,
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SplitTunnelingViewModelState(),
            )

    val uiState =
        vmState
            .map(SplitTunnelingViewModelState::toUiState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SplitTunnelingUiState.Loading(enabled = false),
            )

    init {
        viewModelScope.launch(dispatcher) {
            val result = splitTunnelingRepository.tryMigrateSplitTunneling()
            if (result is MigrateSplitTunnelingResult.Failed) {
                Log.e("SplitTunnelingViewModel", "Could not migrate split tunneling")
                // Show error to user
                _uiSideEffect.send(SplitTunnelingUiSideEffect.ShowMigrateSplitTunnelingError)
            }
            fetchApps()
        }
    }

    fun onEnableSplitTunneling(isEnabled: Boolean) {
        viewModelScope.launch(dispatcher) {
            splitTunnelingRepository.enableSplitTunneling(isEnabled)
        }
    }

    fun onIncludeAppClick(packageName: String) {
        viewModelScope.launch(dispatcher) { splitTunnelingRepository.includeApp(packageName) }
    }

    fun onExcludeAppClick(packageName: String) {
        viewModelScope.launch(dispatcher) { splitTunnelingRepository.excludeApp(packageName) }
    }

    fun onShowSystemAppsClick(show: Boolean) {
        viewModelScope.launch(dispatcher) { showSystemApps.emit(show) }
    }

    private suspend fun fetchApps() {
        appsProvider.getAppsList().let { appsList -> allApps.emit(appsList) }
    }
}

sealed interface SplitTunnelingUiSideEffect {
    data object ShowMigrateSplitTunnelingError : SplitTunnelingUiSideEffect
}

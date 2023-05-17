package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import net.mullvad.mullvadvpn.ui.serviceconnection.SplitTunneling

class SplitTunnelingViewModel(
    private val appsProvider: ApplicationsProvider,
    private val splitTunneling: SplitTunneling,
    private val dispatcher: CoroutineDispatcher
) : ViewModel() {

    private val allApps = MutableStateFlow<List<AppData>?>(null)
    private val showSystemApps = MutableStateFlow(false)

    private val vmState =
        combine(splitTunneling.excludedAppsCallbackFlow(), allApps, showSystemApps) {
                excludedApps,
                allApps,
                showSystemApps ->
                SplitTunnelingViewModelState(
                    excludedApps = excludedApps,
                    allApps = allApps,
                    showSystemApps = showSystemApps
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SplitTunnelingViewModelState()
            )

    val uiState =
        vmState
            .map(SplitTunnelingViewModelState::toUiState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SplitTunnelingUiState.Loading
            )

    init {
        viewModelScope.launch(dispatcher) {
            if (!splitTunneling.enabled) splitTunneling.enabled = true
            fetchApps()
        }
    }

    override fun onCleared() {
        splitTunneling.persist()
        super.onCleared()
    }
    
    fun onIncludeAppClick(packageName: String) {
        viewModelScope.launch(dispatcher) { splitTunneling.includeApp(packageName) }
    }

    fun onExcludeAppClick(packageName: String) {
        viewModelScope.launch(dispatcher) { splitTunneling.excludeApp(packageName) }
    }

    fun onShowSystemAppsClicked(show: Boolean) {
        viewModelScope.launch(dispatcher) { showSystemApps.emit(show) }
    }

    private suspend fun fetchApps() {
        appsProvider.getAppsList().let { appsList -> allApps.emit(appsList) }
    }

    private fun SplitTunneling.excludedAppsCallbackFlow() = callbackFlow {
        excludedAppsChange = { apps -> trySend(apps) }
        awaitClose { emptySet<String>() }
    }
}

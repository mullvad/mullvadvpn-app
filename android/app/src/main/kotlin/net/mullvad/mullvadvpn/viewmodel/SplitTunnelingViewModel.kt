package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState

// import net.mullvad.mullvadvpn.ui.serviceconnection.splitTunneling

class SplitTunnelingViewModel(
    private val appsProvider: ApplicationsProvider,
    private val dispatcher: CoroutineDispatcher
) : ViewModel() {

    private val allApps = MutableStateFlow<List<AppData>?>(null)
    private val showSystemApps = MutableStateFlow(false)

    private val vmState: StateFlow<SplitTunnelingViewModelState> = TODO()
    //        combine(
    //                serviceConnection.splitTunneling.excludedAppsCallbackFlow(),
    //                serviceConnection.splitTunneling.enabledCallbackFlow(),
    //                allApps,
    //                showSystemApps,
    //            ) { excludedApps, enabled, allApps, showSystemApps ->
    //                SplitTunnelingViewModelState(
    //                    excludedApps = excludedApps,
    //                    enabled = enabled,
    //                    allApps = allApps,
    //                    showSystemApps = showSystemApps
    //                )
    //            }
    //            .stateIn(
    //                viewModelScope,
    //                SharingStarted.WhileSubscribed(),
    //                SplitTunnelingViewModelState()
    //            )

    val uiState =
        vmState
            .map(SplitTunnelingViewModelState::toUiState)
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                SplitTunnelingUiState.Loading(enabled = false)
            )

    init {
        viewModelScope.launch(dispatcher) { fetchApps() }
    }

    override fun onCleared() {
        //        serviceConnectionManager.splitTunneling()?.persist()
        super.onCleared()
    }

    fun onEnableSplitTunneling(isEnabled: Boolean) {
        viewModelScope.launch(dispatcher) {
            //            serviceConnectionManager.splitTunneling()?.enableSplitTunneling(isEnabled)
        }
    }

    fun onIncludeAppClick(packageName: String) {
        viewModelScope.launch(dispatcher) {
            //            serviceConnectionManager.splitTunneling()?.includeApp(packageName)
        }
    }

    fun onExcludeAppClick(packageName: String) {
        viewModelScope.launch(dispatcher) {
            //            serviceConnectionManager.splitTunneling()?.excludeApp(packageName)
        }
    }

    fun onShowSystemAppsClick(show: Boolean) {
        viewModelScope.launch(dispatcher) { showSystemApps.emit(show) }
    }

    private suspend fun fetchApps() {
        appsProvider.getAppsList().let { appsList -> allApps.emit(appsList) }
    }

    //    private fun SplitTunneling.excludedAppsCallbackFlow() = callbackFlow {
    //        excludedAppsChange = { apps -> trySend(apps) }
    //        awaitClose { emptySet<String>() }
    //    }
    //
    //    private fun SplitTunneling.enabledCallbackFlow() = callbackFlow {
    //        enabledChange = { isEnabled -> trySend(isEnabled) }
    //        awaitClose()
    //    }
}

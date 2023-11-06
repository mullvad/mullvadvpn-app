package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.SplitTunneling
import net.mullvad.mullvadvpn.ui.serviceconnection.splitTunneling

class SplitTunnelingViewModel(
    private val appsProvider: ApplicationsProvider,
    private val serviceConnectionManager: ServiceConnectionManager,
    private val dispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val allApps = MutableStateFlow<List<AppData>?>(null)
    private val showSystemApps = MutableStateFlow(false)

    private val _shared: SharedFlow<ServiceConnectionContainer> =
        serviceConnectionManager.connectionState
            .flatMapLatest { state ->
                if (state is ServiceConnectionState.ConnectedReady) {
                    flowOf(state.container)
                } else {
                    emptyFlow()
                }
            }
            .shareIn(viewModelScope, SharingStarted.WhileSubscribed())

    private val vmState =
        _shared
            .flatMapLatest { serviceConnection ->
                combine(
                    serviceConnection.splitTunneling.excludedAppsCallbackFlow(),
                    allApps,
                    showSystemApps
                ) { excludedApps, allApps, showSystemApps ->
                    SplitTunnelingViewModelState(
                        excludedApps = excludedApps,
                        allApps = allApps,
                        showSystemApps = showSystemApps
                    )
                }
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
            if (serviceConnectionManager.splitTunneling()?.enabled == false) {
                serviceConnectionManager.splitTunneling()?.enabled = true
            }
            fetchApps()
        }
    }

    override fun onCleared() {
        serviceConnectionManager.splitTunneling()?.persist()
        super.onCleared()
    }

    fun onIncludeAppClick(packageName: String) {
        viewModelScope.launch(dispatcher) {
            serviceConnectionManager.splitTunneling()?.includeApp(packageName)
        }
    }

    fun onExcludeAppClick(packageName: String) {
        viewModelScope.launch(dispatcher) {
            serviceConnectionManager.splitTunneling()?.excludeApp(packageName)
        }
    }

    fun onShowSystemAppsClick(show: Boolean) {
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

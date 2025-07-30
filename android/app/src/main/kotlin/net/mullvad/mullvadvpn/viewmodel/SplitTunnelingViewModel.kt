package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.SplitTunnelingDestination
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.lib.model.AppId
import net.mullvad.mullvadvpn.repository.SplitTunnelingRepository
import net.mullvad.mullvadvpn.util.Lc

class SplitTunnelingViewModel(
    private val appsProvider: ApplicationsProvider,
    private val splitTunnelingRepository: SplitTunnelingRepository,
    savedStateHandle: SavedStateHandle,
    private val dispatcher: CoroutineDispatcher,
) : ViewModel() {
    private val navArgs = SplitTunnelingDestination.argsFrom(savedStateHandle)

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
            .map { it.toUiState(navArgs.isModal) }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                Lc.Loading(Loading(enabled = false, isModal = navArgs.isModal)),
            )

    init {
        viewModelScope.launch(dispatcher) { fetchApps() }
    }

    fun onEnableSplitTunneling(isEnabled: Boolean) {
        viewModelScope.launch(dispatcher) {
            splitTunnelingRepository.enableSplitTunneling(isEnabled)
        }
    }

    fun onIncludeAppClick(packageName: String) {
        viewModelScope.launch(dispatcher) {
            splitTunnelingRepository.includeApp(AppId(packageName))
        }
    }

    fun onExcludeAppClick(packageName: String) {
        viewModelScope.launch(dispatcher) {
            splitTunnelingRepository.excludeApp(AppId(packageName))
        }
    }

    fun onShowSystemAppsClick(show: Boolean) {
        viewModelScope.launch(dispatcher) { showSystemApps.emit(show) }
    }

    private suspend fun fetchApps() {
        appsProvider.getAppsList().let { appsList -> allApps.emit(appsList) }
    }
}

data class Loading(val enabled: Boolean = false, val isModal: Boolean = false)

data class SplitTunnelingUiState(
    val enabled: Boolean = false,
    val excludedApps: List<AppData> = emptyList(),
    val includedApps: List<AppData> = emptyList(),
    val showSystemApps: Boolean = false,
    val isModal: Boolean = false,
)

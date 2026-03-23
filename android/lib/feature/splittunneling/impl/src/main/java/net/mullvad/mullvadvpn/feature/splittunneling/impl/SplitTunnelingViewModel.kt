package net.mullvad.mullvadvpn.feature.splittunneling.impl

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.splittunneling.destinations.SplitTunnelingDestination
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.feature.splittunneling.impl.applist.AppData
import net.mullvad.mullvadvpn.feature.splittunneling.impl.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.model.AppId
import net.mullvad.mullvadvpn.lib.model.SplitTunnelMode
import net.mullvad.mullvadvpn.lib.repository.SplitTunnelingRepository

class SplitTunnelingViewModel(
    private val appsProvider: ApplicationsProvider,
    private val splitTunnelingRepository: SplitTunnelingRepository,
    savedStateHandle: SavedStateHandle,
    private val dispatcher: CoroutineDispatcher,
) : ViewModel() {
    private val navArgs = SplitTunnelingDestination.argsFrom(savedStateHandle)

    private val allApps = MutableStateFlow<List<AppData>?>(null)
    private val showSystemApps = MutableStateFlow(false)

    val uiState: StateFlow<Lc<Loading, SplitTunnelingUiState>> =
        combine(
                splitTunnelingRepository.excludedApps,
                splitTunnelingRepository.splitTunnelingEnabled,
                splitTunnelingRepository.splitTunnelingMode,
                allApps,
                showSystemApps,
            ) { excludedApps, enabled, mode, allApps, showSystemApps ->
                if (allApps == null) {
                    return@combine Lc.Loading(Loading(enabled = enabled, isModal = navArgs.isModal))
                }

                // In EXCLUDE mode: apps in the list are shown in the top "selected" section.
                // In INCLUDE mode: apps in the list are shown in the top "selected" section.
                // Either way, "selected" apps are those in the repository list.
                val (selectedApps, unselectedApps) =
                    allApps.partition { appData ->
                        if (enabled) {
                            excludedApps.contains(AppId(appData.packageName))
                        } else {
                            false
                        }
                    }

                SplitTunnelingUiState(
                        enabled = enabled,
                        mode = mode,
                        excludedApps = selectedApps,
                        includedApps =
                            if (showSystemApps) unselectedApps
                            else unselectedApps.filter { appData -> !appData.isSystemApp },
                        showSystemApps = showSystemApps,
                        isModal = navArgs.isModal,
                    )
                    .toLc()
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
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

    fun onSetMode(mode: SplitTunnelMode) {
        viewModelScope.launch(dispatcher) { splitTunnelingRepository.setMode(mode) }
    }

    private suspend fun fetchApps() {
        appsProvider.apps().let { appsList -> allApps.emit(appsList) }
    }
}

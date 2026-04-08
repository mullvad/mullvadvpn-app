package net.mullvad.mullvadvpn.feature.splittunneling.impl

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
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
import net.mullvad.mullvadvpn.lib.repository.SplitTunnelingRepository
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository

class SplitTunnelingViewModel(
    isModal: Boolean,
    private val appsProvider: ApplicationsProvider,
    private val splitTunnelingRepository: SplitTunnelingRepository,
    private val userPreferencesRepository: UserPreferencesRepository,
    private val dispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val allApps = MutableStateFlow<List<AppData>?>(null)

    val uiState: StateFlow<Lc<Loading, SplitTunnelingUiState>> =
        combine(
                splitTunnelingRepository.excludedApps,
                splitTunnelingRepository.splitTunnelingEnabled,
                allApps,
                userPreferencesRepository.showSystemAppsSplitTunneling(),
            ) { excludedApps, enabled, allApps, showSystemApps ->
                if (allApps == null) {
                    return@combine Lc.Loading(Loading(enabled = enabled, isModal = isModal))
                }

                val (excludedApps, includedApps) =
                    allApps.partition { appData ->
                        if (enabled) {
                            excludedApps.contains(AppId(appData.packageName))
                        } else {
                            false
                        }
                    }

                SplitTunnelingUiState(
                        enabled = enabled,
                        excludedApps = excludedApps,
                        includedApps =
                            if (showSystemApps) includedApps
                            else includedApps.filter { appData -> !appData.isSystemApp },
                        showSystemApps = showSystemApps,
                        isModal = isModal,
                    )
                    .toLc()
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Loading(enabled = false, isModal = isModal)),
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
        viewModelScope.launch(dispatcher) {
            userPreferencesRepository.setShowSystemAppsSplitTunneling(show)
        }
    }

    private suspend fun fetchApps() {
        appsProvider.apps().let { appsList -> allApps.emit(appsList) }
    }
}

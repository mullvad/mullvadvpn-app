package net.mullvad.mullvadvpn.feature.splittunneling.impl.search

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

class SearchSplitTunnelingViewModel(
    private val appsProvider: ApplicationsProvider,
    private val splitTunnelingRepository: SplitTunnelingRepository,
    private val userPreferencesRepository: UserPreferencesRepository,
    private val dispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val allApps = MutableStateFlow<List<AppData>?>(null)
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

    val uiState: StateFlow<Lc<Unit, SearchSplitTunnelingUiState>> =
        combine(
                _searchTerm,
                splitTunnelingRepository.excludedApps,
                allApps,
                userPreferencesRepository.showSystemAppsSplitTunneling(),
            ) {
                searchTerm,
                excludedApps,
                allApps,
                showSystemApps ->
                if (allApps == null) {
                    return@combine Lc.Loading(Unit)
                }

                val filteredApps =
                    if (searchTerm.isNotEmpty()) {
                        allApps.filter { it.name.contains(searchTerm, ignoreCase = true) }
                    } else {
                        allApps
                    }

                val (excluded, included) =
                    filteredApps.partition { appData ->
                        excludedApps.contains(AppId(appData.packageName))
                    }

                SearchSplitTunnelingUiState(
                        searchTerm = searchTerm,
                        excludedApps = excluded,
                        includedApps =
                            if (showSystemApps) included
                            else included.filter { appData -> !appData.isSystemApp },
                        showSystemApps = showSystemApps,
                    )
                    .toLc()
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    init {
        viewModelScope.launch(dispatcher) { fetchApps() }
    }

    fun onSearchInputChanged(searchTerm: String) {
        viewModelScope.launch { _searchTerm.emit(searchTerm) }
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

    private suspend fun fetchApps() {
        appsProvider.apps().let { appsList -> allApps.emit(appsList) }
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

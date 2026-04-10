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
import net.mullvad.mullvadvpn.feature.splittunneling.impl.applist.SplitTunnelingUseCase
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.AppId
import net.mullvad.mullvadvpn.lib.repository.SplitTunnelingRepository

class SearchSplitTunnelingViewModel(
    splitTunnelingUseCase: SplitTunnelingUseCase,
    private val splitTunnelingRepository: SplitTunnelingRepository,
    private val dispatcher: CoroutineDispatcher,
) : ViewModel() {
    private val _searchTerm = MutableStateFlow(EMPTY_SEARCH_TERM)

    val uiState: StateFlow<Lc<Unit, SearchSplitTunnelingUiState>> =
        combine(splitTunnelingUseCase(), _searchTerm) { splitApps, searchTerm ->
                Lc.Content(
                    SearchSplitTunnelingUiState(
                        searchTerm = searchTerm,
                        excludedApps =
                            splitApps.excludedApps.filter {
                                it.name.contains(searchTerm, ignoreCase = true)
                            },
                        includedApps =
                            splitApps.includedApps.filter {
                                it.name.contains(searchTerm, ignoreCase = true)
                            },
                    )
                )
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lc.Loading(Unit),
            )

    fun onSearchInputChanged(searchTerm: String) {
        viewModelScope.launch { _searchTerm.emit(searchTerm) }
    }

    fun onIncludeAppClick(packageName: AppId) {
        viewModelScope.launch(dispatcher) { splitTunnelingRepository.includeApp(packageName) }
    }

    fun onExcludeAppClick(packageName: AppId) {
        viewModelScope.launch(dispatcher) { splitTunnelingRepository.excludeApp(packageName) }
    }

    companion object {
        private const val EMPTY_SEARCH_TERM = ""
    }
}

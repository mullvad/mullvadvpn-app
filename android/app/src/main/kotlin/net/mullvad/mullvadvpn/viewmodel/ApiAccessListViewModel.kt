package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.ApiAccessListUiState
import net.mullvad.mullvadvpn.repository.ApiAccessRepository

class ApiAccessListViewModel(apiAccessRepository: ApiAccessRepository) : ViewModel() {

    val uiState =
        apiAccessRepository.accessMethods
            .map { apiAccessMethods ->
                ApiAccessListUiState(
                    apiAccessMethods?.firstOrNull(),
                    apiAccessMethods = apiAccessMethods ?: emptyList()
                )
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), ApiAccessListUiState())
}

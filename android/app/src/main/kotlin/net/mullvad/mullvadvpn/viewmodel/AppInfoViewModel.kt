package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.repository.ChangelogRepository
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoRepository

class AppInfoViewModel(
    changelogRepository: ChangelogRepository,
    appVersionInfoRepository: AppVersionInfoRepository,
    isPlayBuild: Boolean,
) : ViewModel() {

    val uiState: StateFlow<AppInfoUiState> =
        combine(
                appVersionInfoRepository.versionInfo,
                flowOf(changelogRepository.getLastVersionChanges()),
                flowOf(isPlayBuild),
            ) { versionInfo, changes, isPlayBuild ->
                AppInfoUiState(versionInfo, changes, isPlayBuild)
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                AppInfoUiState(
                    appVersionInfoRepository.versionInfo.value,
                    changelogRepository.getLastVersionChanges(),
                    true,
                ),
            )
}

data class AppInfoUiState(
    val version: VersionInfo,
    val changes: List<String>,
    val isPlayBuild: Boolean,
)

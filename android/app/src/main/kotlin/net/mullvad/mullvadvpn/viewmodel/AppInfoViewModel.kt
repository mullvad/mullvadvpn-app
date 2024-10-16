package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.repository.ChangelogRepository
import net.mullvad.mullvadvpn.ui.VersionInfo
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoRepository

class AppInfoViewModel(
    changelogRepository: ChangelogRepository,
    appVersionInfoRepository: AppVersionInfoRepository,
    val resources: Resources,
    val isPlayBuild: Boolean,
    val packageName: String,
) : ViewModel() {

    private val _uiSideEffect = Channel<AppInfoSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

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

    fun openAppListing() =
        viewModelScope.launch {
            val uri =
                if (isPlayBuild) {
                    resources.getString(R.string.market_uri, packageName)
                } else {
                    resources.getString(R.string.download_url)
                }
            _uiSideEffect.send(AppInfoSideEffect.OpenUri(Uri.parse(uri)))
        }
}

data class AppInfoUiState(
    val version: VersionInfo,
    val changes: List<String>,
    val isPlayBuild: Boolean,
)

sealed interface AppInfoSideEffect {
    data class OpenUri(val uri: Uri) : AppInfoSideEffect
}

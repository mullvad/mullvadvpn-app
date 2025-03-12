package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import android.net.Uri
import androidx.core.net.toUri
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
    private val resources: Resources,
    private val isPlayBuild: Boolean,
    private val isFdroidBuild: Boolean,
    private val packageName: String,
) : ViewModel() {

    private val _uiSideEffect = Channel<AppInfoSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<AppInfoUiState> =
        combine(
                appVersionInfoRepository.versionInfo,
                flowOf(changelogRepository.getLastVersionChanges()),
                flowOf(isPlayBuild),
            ) { versionInfo, changes, isPlayBuild ->
                AppInfoUiState(versionInfo, isPlayBuild)
            }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(),
                AppInfoUiState(appVersionInfoRepository.versionInfo.value, true),
            )

    fun openAppListing() =
        viewModelScope.launch {
            val uri =
                if (isPlayBuild || isFdroidBuild) {
                    resources.getString(R.string.market_uri, packageName)
                } else {
                    resources.getString(R.string.download_url)
                }
            _uiSideEffect.send(AppInfoSideEffect.OpenUri(uri.toUri()))
        }
}

data class AppInfoUiState(val version: VersionInfo, val isPlayBuild: Boolean)

sealed interface AppInfoSideEffect {
    data class OpenUri(val uri: Uri) : AppInfoSideEffect
}

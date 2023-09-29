package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.mullvad.mullvadvpn.repository.ChangelogRepository

class ChangelogViewModel(
    private val changelogRepository: ChangelogRepository,
    private val buildVersionCode: Int,
    private val alwaysShowChangelog: Boolean
) : ViewModel() {
    private val _uiState = MutableStateFlow<ChangelogDialogUiState>(ChangelogDialogUiState.Hide)
    val uiState = _uiState.asStateFlow()

    fun refreshChangelogDialogUiState() {
        val shouldShowChangelogDialog =
            alwaysShowChangelog ||
                changelogRepository.getVersionCodeOfMostRecentChangelogShowed() < buildVersionCode
        _uiState.value =
            if (shouldShowChangelogDialog) {
                val changelogList = changelogRepository.getLastVersionChanges()
                if (changelogList.isNotEmpty()) {
                    ChangelogDialogUiState.Show(changelogList)
                } else {
                    ChangelogDialogUiState.Hide
                }
            } else {
                ChangelogDialogUiState.Hide
            }
    }

    fun dismissChangelogDialog() {
        changelogRepository.setVersionCodeOfMostRecentChangelogShowed(buildVersionCode)
        _uiState.value = ChangelogDialogUiState.Hide
    }
}

sealed class ChangelogDialogUiState {
    data class Show(val changes: List<String>) : ChangelogDialogUiState()

    data object Hide : ChangelogDialogUiState()
}

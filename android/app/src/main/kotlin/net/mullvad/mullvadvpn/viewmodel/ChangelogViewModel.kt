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
    private val _changelogDialogUiState =
        MutableStateFlow<ChangelogDialogUiState>(ChangelogDialogUiState.Hide)
    val changelogDialogUiState = _changelogDialogUiState.asStateFlow()

    fun refreshChangelogDialogUiState() {
        val shouldShowChangelogDialog = alwaysShowChangelog || changelogRepository
            .getVersionCodeOfMostRecentChangelogShowed() < buildVersionCode
        _changelogDialogUiState.value = if (shouldShowChangelogDialog) {
            ChangelogDialogUiState.Show(changelogRepository.getLastVersionChanges())
        } else {
            ChangelogDialogUiState.Hide
        }
    }

    fun dismissChangelogDialog() {
        changelogRepository.setVersionCodeOfMostRecentChangelogShowed(buildVersionCode)
        _changelogDialogUiState.value = ChangelogDialogUiState.Hide
    }
}

sealed class ChangelogDialogUiState {
    data class Show(val changes: List<String>) : ChangelogDialogUiState()
    object Hide : ChangelogDialogUiState()
}

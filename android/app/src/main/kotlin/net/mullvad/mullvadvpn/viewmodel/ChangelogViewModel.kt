package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.repository.AppChangesRepository

class ChangelogViewModel(
    private val appChangesRepository: AppChangesRepository
) : ViewModel() {
    private val _changelogDialogUiState =
        MutableStateFlow<ChangelogDialogUiState>(ChangelogDialogUiState.Hide)
    val changelogDialogUiState = _changelogDialogUiState.asStateFlow()

    fun refreshChangelogDialogUiState() {
        val shouldShowChangelogDialog = BuildConfig.ALWAYS_SHOW_CHANGELOG || appChangesRepository
            .getVersionCodeOfMostRecentChangelogShowed() < BuildConfig.VERSION_CODE
        _changelogDialogUiState.value = if (shouldShowChangelogDialog) {
            ChangelogDialogUiState.Show(appChangesRepository.getLastVersionChanges())
        } else {
            ChangelogDialogUiState.Hide
        }
    }

    fun dismissChangelogDialog() {
        appChangesRepository.setVersionCodeOfMostRecentChangelogShowed(BuildConfig.VERSION_CODE)
        _changelogDialogUiState.value = ChangelogDialogUiState.Hide
    }
}

sealed class ChangelogDialogUiState {
    data class Show(val changes: List<String>) : ChangelogDialogUiState()
    object Hide : ChangelogDialogUiState()
}

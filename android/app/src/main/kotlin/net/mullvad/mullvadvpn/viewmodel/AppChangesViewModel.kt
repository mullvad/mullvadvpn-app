package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.repository.AppChangesRepository

class AppChangesViewModel(
    private val appChangesRepository: AppChangesRepository,
    dispatcher: CoroutineDispatcher = Dispatchers.Default
) : ViewModel() {
    private val shouldShowChangelog = MutableStateFlow(appChangesRepository.shouldShowLastChanges())

    val changeLogUiState = shouldShowChangelog
        .map { shouldShow ->
            if (shouldShow) {
                ChangelogDialogState.Show(appChangesRepository.getLastVersionChanges())
            } else {
                ChangelogDialogState.Hide
            }
        }
        .stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.WhileSubscribed(),
            ChangelogDialogState.Hide
        )

    fun refresh() {
        shouldShowChangelog.value = appChangesRepository.shouldShowLastChanges().let {
            true
        }
    }

    fun setDialogShowed() = appChangesRepository.setShowedLastChanges().also {
        shouldShowChangelog.value = false
    }
}

sealed class ChangelogDialogState {
    data class Show(val changes: List<String>) : ChangelogDialogState()
    object Hide : ChangelogDialogState()

    var changelogList: List<String> = emptyList()
        get() {
            return if (this is Show) this.changes else emptyList()
        }
        private set
}

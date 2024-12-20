package net.mullvad.mullvadvpn.viewmodel

import android.os.Parcelable
import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.repository.ChangelogRepository

class ChangelogViewModel(
    private val changelogRepository: ChangelogRepository,
    buildVersion: BuildVersion,
) : ViewModel() {
    val uiState: StateFlow<ChangelogUiState> =
        MutableStateFlow(
            ChangelogUiState(buildVersion.name, changelogRepository.getLastVersionChanges())
        )

    fun dismissChangelogNotification() {
        changelogRepository.setDismissNewChangelogNotification()
    }
}

@Parcelize data class ChangelogUiState(val version: String, val changes: List<String>) : Parcelable

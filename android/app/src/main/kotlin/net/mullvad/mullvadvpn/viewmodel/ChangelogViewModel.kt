package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.repository.ChangelogRepository

class ChangelogViewModel(
    private val changelogRepository: ChangelogRepository,
    private val buildVersion: BuildVersion,
) : ViewModel() {

    fun markChangelogAsRead() {
        changelogRepository.setVersionCodeOfMostRecentChangelogShowed(buildVersion.code)
    }
}

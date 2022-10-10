package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import net.mullvad.mullvadvpn.repository.IAppChangesRepository

class AppChangesViewModel(
    private val appChangesRepository: IAppChangesRepository
) : ViewModel() {
    fun resetShouldShowChanges() = appChangesRepository.resetShouldShowLastChanges()
    fun shouldShowChanges() = appChangesRepository.shouldShowLastChanges()
    fun setDialogShowed() = appChangesRepository.setShowedLastChanges()
    fun getChangesList() = appChangesRepository.getLastVersionChanges()
}

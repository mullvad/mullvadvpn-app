package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.repository.AppChangesRepository
import net.mullvad.mullvadvpn.repository.ChangeLogState

class AppChangesViewModel(
    private val appChangesRepository: AppChangesRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.Default
) : ViewModel() {
    private val cachedChangeLogState = MutableStateFlow(
        if (appChangesRepository.shouldShowLastChanges())
            ChangeLogState.ShouldShow
        else ChangeLogState.AlreadyShowed
    )

    val changeLogState = cachedChangeLogState
        .flatMapLatest { state ->
            if (state == ChangeLogState.ShouldShow)
                flowOf(state)
            flowOf(state)
        }
        .stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.WhileSubscribed(),
            ChangeLogState.ShouldShow
        )

    fun resetShouldShowChanges() = appChangesRepository.resetShouldShowLastChanges().also {
        cachedChangeLogState.value = ChangeLogState.ShouldShow
    }
    fun shouldShowChanges() = appChangesRepository.shouldShowLastChanges()
    fun setDialogShowed() = appChangesRepository.setShowedLastChanges().also {
        cachedChangeLogState.value = ChangeLogState.AlreadyShowed
    }

    fun getChangesList() = appChangesRepository.getLastVersionChanges()
}

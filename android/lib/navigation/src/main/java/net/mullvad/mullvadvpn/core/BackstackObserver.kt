package net.mullvad.mullvadvpn.core

import androidx.navigation.NavController
import androidx.navigation.NavDestination
import androidx.navigation.NavHostController
import androidx.savedstate.SavedState
import com.ramcosta.composedestinations.spec.DestinationSpec
import com.ramcosta.composedestinations.utils.destination
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

class BackstackObserver : NavController.OnDestinationChangedListener {

    private val _destinationFlow: MutableStateFlow<DestinationSpec?> = MutableStateFlow(null)

    private val _previousDestinationFlow: MutableStateFlow<DestinationSpec?> =
        MutableStateFlow(null)

    val destinationFlow: StateFlow<DestinationSpec?> = _destinationFlow

    val previousDestinationFlow: StateFlow<DestinationSpec?> = _previousDestinationFlow

    fun addOnDestinationChangedListener(navHostController: NavHostController) {
        navHostController.addOnDestinationChangedListener(this)
    }

    fun removeOnDestinationChangedListener(navHostController: NavHostController) {
        navHostController.removeOnDestinationChangedListener(this)
    }

    override fun onDestinationChanged(
        controller: NavController,
        destination: NavDestination,
        arguments: SavedState?,
    ) {
        _previousDestinationFlow.value = _destinationFlow.value
        controller.currentBackStackEntry?.destination()?.let { _destinationFlow.value = it }
    }
}

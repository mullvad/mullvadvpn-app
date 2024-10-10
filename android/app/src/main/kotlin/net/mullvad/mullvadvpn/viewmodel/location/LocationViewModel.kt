package net.mullvad.mullvadvpn.viewmodel.location

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.RelayItemId

// Common behaviour for SearchLocationViewModel and SelectLocationViewModel
abstract class LocationViewModel : ViewModel() {
}

package net.mullvad.mullvadvpn.compose.bottomsheet

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.bottomsheet.spec.DestinationStyleBottomSheet
import com.ramcosta.composedestinations.result.ResultBackNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.communication.LocationsChanged
import net.mullvad.mullvadvpn.compose.component.MullvadModalBottomSheet
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.viewmodel.CustomListEntrySheetSideEffect
import net.mullvad.mullvadvpn.viewmodel.CustomListEntrySheetViewModel
import org.koin.androidx.compose.koinViewModel

data class CustomListEntrySheetNavArgs(
    val name: String,
    val customListId: CustomListId,
    val location: GeoLocationId
)

@OptIn(ExperimentalMaterial3Api::class)
@Destination<RootGraph>(
    navArgs = CustomListEntrySheetNavArgs::class,
    style = DestinationStyleBottomSheet::class
)
@Composable
fun CustomListEntrySheet(backNavigator: ResultBackNavigator<LocationsChanged>) {
    val vm = koinViewModel<CustomListEntrySheetViewModel>()
    val state = vm.uiState.collectAsStateWithLifecycle()
    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            CustomListEntrySheetSideEffect.GenericError -> TODO("How do we handle error?")
            is CustomListEntrySheetSideEffect.LocationRemovedFromCustomList ->
                backNavigator.navigateBack(it.locationsChanged)
        }
    }
    MullvadModalBottomSheet {
        HeaderCell(
            text =
                stringResource(id = R.string.remove_location_from_list, state.value.locationName),
            background = Color.Unspecified
        )
        HorizontalDivider(color = MaterialTheme.colorScheme.onBackground)

        IconCell(
            iconId = R.drawable.ic_remove,
            title = stringResource(id = R.string.remove_button),
            titleColor = MaterialTheme.colorScheme.onBackground,
            onClick = vm::removeLocationFromList,
            background = Color.Unspecified
        )
    }
}

package net.mullvad.mullvadvpn.compose.bottomsheet

import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.bottomsheet.spec.DestinationStyleBottomSheet
import com.ramcosta.composedestinations.generated.destinations.CreateCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.LocationSheetDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import kotlin.collections.forEach
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.communication.CustomListSuccess
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.compose.component.MullvadModalBottomContainer
import net.mullvad.mullvadvpn.compose.state.LocationUiState
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.viewmodel.LocationSheetViewModel
import net.mullvad.mullvadvpn.viewmodel.LocationSideEffect
import org.koin.androidx.compose.koinViewModel

data class LocationNavArgs(val locationName: String, val id: GeoLocationId)

@OptIn(ExperimentalMaterial3Api::class)
@Destination<RootGraph>(
    navArgs = LocationNavArgs::class,
    style = DestinationStyleBottomSheet::class
)
@Composable
fun LocationSheet(
    navigator: DestinationsNavigator,
    backNavigator: ResultBackNavigator<CustomListSuccess>,
) {
    val viewModel = koinViewModel<LocationSheetViewModel>()
    val state = viewModel.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            LocationSideEffect.GenericError -> {
                TODO("Handle")
            }
            is LocationSideEffect.LocationAddedToCustomList ->
                backNavigator.navigateBack(it.locationsChanged)
        }
    }

    MullvadModalBottomContainer {
        HeaderCell(
            text = stringResource(id = R.string.add_location_to_list, state.value.name),
            background = Color.Unspecified
        )
        HorizontalDivider(color = MaterialTheme.colorScheme.onBackground)

        when (val s = state.value) {
            is LocationUiState.Content ->
                LocationContent(
                    s,
                    {
                        navigator.navigate(CreateCustomListDestination(s.location.id)) {
                            popUpTo(LocationSheetDestination) { inclusive = true }
                        }
                    },
                    viewModel::addLocationToList
                )
            is LocationUiState.Loading ->
                MullvadCircularProgressIndicatorMedium(
                    modifier = Modifier.align(Alignment.CenterHorizontally)
                )
        }
    }
}

@Composable
private fun ColumnScope.LocationContent(
    state: LocationUiState.Content,
    createCustomListWithLocation: (location: GeoLocationId) -> Unit,
    addLocationToList: (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit
) {
    state.customLists.forEach {
        IconCell(
            iconId = null,
            title =
                if (it.canAdd) {
                    it.customList.name
                } else {
                    stringResource(id = R.string.location_added, it.customList.name)
                },
            titleColor =
                if (it.canAdd) {
                    MaterialTheme.colorScheme.onBackground
                } else {
                    MaterialTheme.colorScheme.onSecondary
                },
            onClick = { addLocationToList(state.location, it.customList) },
            background = Color.Unspecified,
            enabled = it.canAdd
        )
    }
    IconCell(
        iconId = R.drawable.icon_add,
        title = stringResource(id = R.string.new_list),
        titleColor = MaterialTheme.colorScheme.onBackground,
        onClick = { createCustomListWithLocation(state.location.id) },
        background = Color.Unspecified
    )
}

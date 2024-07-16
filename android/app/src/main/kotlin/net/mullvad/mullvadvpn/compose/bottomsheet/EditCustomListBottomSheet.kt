package net.mullvad.mullvadvpn.compose.bottomsheet

import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.bottomsheet.spec.DestinationStyleBottomSheet
import com.ramcosta.composedestinations.generated.destinations.CustomListLocationsDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListSheetDestination
import com.ramcosta.composedestinations.generated.destinations.DeleteCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.EditCustomListNameDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.component.MullvadModalBottomSheet
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.viewmodel.CustomListSheetUiState
import net.mullvad.mullvadvpn.viewmodel.CustomListSheetViewModel
import org.koin.androidx.compose.koinViewModel

data class CustomListSheetNavArgs(
    val customListId: CustomListId,
    val customListName: CustomListName
)

@OptIn(ExperimentalMaterial3Api::class)
@Destination<RootGraph>(
    navArgs = CustomListSheetNavArgs::class,
    style = DestinationStyleBottomSheet::class
)
@Composable
fun CustomListSheet(
    navigator: DestinationsNavigator,
) {
    val vm = koinViewModel<CustomListSheetViewModel>()
    val state = vm.uiState.collectAsStateWithLifecycle()
    MullvadModalBottomSheet {
        CustomListContent(
            state.value,
            dropUnlessResumed {
                navigator.navigate(
                    EditCustomListNameDestination(
                        state.value.customListId,
                        state.value.customListName
                    )
                ) {
                    popUpTo(CustomListSheetDestination) { inclusive = true }
                }
            },
            dropUnlessResumed {
                navigator.navigate(
                    CustomListLocationsDestination(state.value.customListId, false)
                ) {
                    popUpTo(CustomListSheetDestination) { inclusive = true }
                }
            },
            dropUnlessResumed {
                navigator.navigate(
                    DeleteCustomListDestination(
                        state.value.customListId,
                        state.value.customListName,
                    )
                ) {
                    popUpTo(CustomListSheetDestination) { inclusive = true }
                }
            },
        )
    }
}

@Composable
private fun ColumnScope.CustomListContent(
    state: CustomListSheetUiState,
    editCustomListName: () -> Unit,
    editLocations: () -> Unit,
    deleteCustomList: () -> Unit,
) {
    HeaderCell(text = state.customListName.value, background = Color.Unspecified)
    IconCell(
        iconId = R.drawable.icon_edit,
        title = stringResource(id = R.string.edit_name),
        titleColor = MaterialTheme.colorScheme.onBackground,
        onClick = editCustomListName,
        background = Color.Unspecified
    )
    IconCell(
        iconId = R.drawable.icon_add,
        title = stringResource(id = R.string.edit_locations),
        titleColor = MaterialTheme.colorScheme.onBackground,
        onClick = editLocations,
        background = Color.Unspecified
    )
    HorizontalDivider(color = MaterialTheme.colorScheme.onBackground)
    IconCell(
        iconId = R.drawable.icon_delete,
        title = stringResource(id = R.string.delete),
        titleColor = MaterialTheme.colorScheme.onBackground,
        onClick = deleteCustomList,
        background = Color.Unspecified
    )
}

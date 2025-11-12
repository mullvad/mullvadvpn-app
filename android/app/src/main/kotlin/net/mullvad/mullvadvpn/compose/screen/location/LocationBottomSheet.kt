package net.mullvad.mullvadvpn.compose.screen.location

import android.content.Context
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SheetState
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import com.ramcosta.composedestinations.spec.DestinationSpec
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.component.MullvadModalBottomSheet
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowCustomListsEntryBottomSheet
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowEditCustomListBottomSheet
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState.ShowLocationBottomSheet
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.relaylist.canAddLocation

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun LocationBottomSheets(
    locationBottomSheetState: LocationBottomSheetState?,
    onCreateCustomList: (RelayItem.Location?) -> Unit,
    onAddLocationToList: (RelayItem.Location, RelayItem.CustomList) -> Unit,
    onRemoveLocationFromList: (location: RelayItem.Location, parent: CustomListId) -> Unit,
    onEditCustomListName: (RelayItem.CustomList) -> Unit,
    onEditLocationsCustomList: (RelayItem.CustomList) -> Unit,
    onDeleteCustomList: (RelayItem.CustomList) -> Unit,
    onHideBottomSheet: () -> Unit,
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()
    val onCloseBottomSheet: (animate: Boolean) -> Unit = { animate ->
        if (animate) {
            scope.launch { sheetState.hide() }.invokeOnCompletion { onHideBottomSheet() }
        } else {
            onHideBottomSheet()
        }
    }
    val backgroundColor: Color = MaterialTheme.colorScheme.surfaceContainer
    val onBackgroundColor: Color = MaterialTheme.colorScheme.onSurface

    when (locationBottomSheetState) {
        is ShowLocationBottomSheet -> {
            LocationBottomSheet(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                sheetState = sheetState,
                customLists = locationBottomSheetState.customLists,
                item = locationBottomSheetState.item,
                onCreateCustomList = onCreateCustomList,
                onAddLocationToList = onAddLocationToList,
                closeBottomSheet = onCloseBottomSheet,
            )
        }
        is ShowEditCustomListBottomSheet -> {
            EditCustomListBottomSheet(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                sheetState = sheetState,
                customList = locationBottomSheetState.customList,
                onEditName = onEditCustomListName,
                onEditLocations = onEditLocationsCustomList,
                onDeleteCustomList = onDeleteCustomList,
                closeBottomSheet = onCloseBottomSheet,
            )
        }
        is ShowCustomListsEntryBottomSheet -> {
            CustomListEntryBottomSheet(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                sheetState = sheetState,
                customListId = locationBottomSheetState.customListId,
                customListName = locationBottomSheetState.customListName,
                item = locationBottomSheetState.item,
                onRemoveLocationFromList = onRemoveLocationFromList,
                closeBottomSheet = onCloseBottomSheet,
            )
        }
        null -> {
            /* Do nothing */
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun LocationBottomSheet(
    backgroundColor: Color,
    onBackgroundColor: Color,
    sheetState: SheetState,
    customLists: List<RelayItem.CustomList>,
    item: RelayItem.Location,
    onCreateCustomList: (relayItem: RelayItem.Location) -> Unit,
    onAddLocationToList: (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit,
) {
    MullvadModalBottomSheet(
        sheetState = sheetState,
        backgroundColor = backgroundColor,
        onBackgroundColor = onBackgroundColor,
        onDismissRequest = { closeBottomSheet(false) },
        modifier = Modifier.testTag(SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG),
    ) { ->
        HeaderCell(
            text = stringResource(id = R.string.add_location_to_list, item.name),
            background = backgroundColor,
        )
        HorizontalDivider(color = onBackgroundColor)
        customLists.forEach {
            val enabled = it.canAddLocation(item)
            IconCell(
                imageVector = null,
                title =
                    if (enabled) {
                        it.name
                    } else {
                        stringResource(id = R.string.location_added, it.name)
                    },
                titleColor =
                    if (enabled) {
                        onBackgroundColor
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant
                    },
                onClick = {
                    onAddLocationToList(item, it)
                    closeBottomSheet(true)
                },
                enabled = enabled,
            )
        }
        IconCell(
            imageVector = Icons.Default.Add,
            title = stringResource(id = R.string.new_list),
            titleColor = onBackgroundColor,
            onClick = {
                onCreateCustomList(item)
                closeBottomSheet(true)
            },
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun EditCustomListBottomSheet(
    backgroundColor: Color,
    onBackgroundColor: Color,
    sheetState: SheetState,
    customList: RelayItem.CustomList,
    onEditName: (item: RelayItem.CustomList) -> Unit,
    onEditLocations: (item: RelayItem.CustomList) -> Unit,
    onDeleteCustomList: (item: RelayItem.CustomList) -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit,
) {
    MullvadModalBottomSheet(
        backgroundColor = backgroundColor,
        onBackgroundColor = onBackgroundColor,
        sheetState = sheetState,
        onDismissRequest = { closeBottomSheet(false) },
        modifier = Modifier.testTag(SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG),
    ) {
        HeaderCell(text = customList.name, background = backgroundColor)
        HorizontalDivider(color = onBackgroundColor)
        IconCell(
            imageVector = Icons.Default.Edit,
            title = stringResource(id = R.string.edit_name),
            titleColor = onBackgroundColor,
            onClick = {
                onEditName(customList)
                closeBottomSheet(true)
            },
        )
        IconCell(
            imageVector = Icons.Default.Add,
            title = stringResource(id = R.string.edit_locations),
            titleColor = onBackgroundColor,
            onClick = {
                onEditLocations(customList)
                closeBottomSheet(true)
            },
        )
        IconCell(
            imageVector = Icons.Default.Delete,
            title = stringResource(id = R.string.delete),
            titleColor = onBackgroundColor,
            onClick = {
                onDeleteCustomList(customList)
                closeBottomSheet(true)
            },
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CustomListEntryBottomSheet(
    backgroundColor: Color,
    onBackgroundColor: Color,
    sheetState: SheetState,
    customListId: CustomListId,
    customListName: CustomListName,
    item: RelayItem.Location,
    onRemoveLocationFromList: (location: RelayItem.Location, customListId: CustomListId) -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit,
) {
    MullvadModalBottomSheet(
        sheetState = sheetState,
        backgroundColor = backgroundColor,
        onBackgroundColor = onBackgroundColor,
        onDismissRequest = { closeBottomSheet(false) },
        modifier = Modifier.testTag(SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG),
    ) {
        HeaderCell(
            text =
                stringResource(id = R.string.remove_location_from_list, item.name, customListName),
            background = backgroundColor,
        )
        HorizontalDivider(color = onBackgroundColor)

        IconCell(
            imageVector = Icons.Default.Remove,
            title = stringResource(id = R.string.remove_button),
            titleColor = onBackgroundColor,
            onClick = {
                onRemoveLocationFromList(item, customListId)
                closeBottomSheet(true)
            },
        )
    }
}

internal suspend fun SnackbarHostState.showResultSnackbar(
    context: Context,
    result: CustomListActionResultData,
    onUndo: (CustomListAction) -> Unit,
) {

    showSnackbarImmediately(
        message = result.message(context),
        actionLabel =
            if (result is CustomListActionResultData.Success) context.getString(R.string.undo)
            else {
                null
            },
        duration = SnackbarDuration.Long,
        onAction = {
            if (result is CustomListActionResultData.Success) {
                onUndo(result.undo)
            }
        },
    )
}

private fun CustomListActionResultData.message(context: Context): String =
    when (this) {
        is CustomListActionResultData.Success.CreatedWithLocations ->
            if (locationNames.size == 1) {
                context.getString(
                    R.string.location_was_added_to_list,
                    locationNames.first(),
                    customListName,
                )
            } else {
                context.getString(R.string.create_custom_list_message, customListName)
            }
        is CustomListActionResultData.Success.Deleted ->
            context.getString(R.string.delete_custom_list_message, customListName)
        is CustomListActionResultData.Success.LocationAdded ->
            context.getString(R.string.location_was_added_to_list, locationName, customListName)
        is CustomListActionResultData.Success.LocationRemoved ->
            context.getString(R.string.location_was_removed_from_list, locationName, customListName)
        is CustomListActionResultData.Success.LocationChanged ->
            context.getString(R.string.locations_were_changed_for, customListName)
        is CustomListActionResultData.Success.Renamed ->
            context.getString(R.string.name_was_changed_to, newName)
        CustomListActionResultData.GenericError -> context.getString(R.string.error_occurred)
    }

@Composable
internal fun <D : DestinationSpec, R : CustomListActionResultData> ResultRecipient<D, R>
    .OnCustomListNavResult(
    snackbarHostState: SnackbarHostState,
    performAction: (action: CustomListAction) -> Unit,
) {
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    this.onNavResult { result ->
        when (result) {
            NavResult.Canceled -> {
                /* Do nothing */
            }
            is NavResult.Value -> {
                // Handle result
                scope.launch {
                    snackbarHostState.showResultSnackbar(
                        context = context,
                        result = result.value,
                        onUndo = performAction,
                    )
                }
            }
        }
    }
}

sealed interface LocationBottomSheetState {
    data class ShowCustomListsEntryBottomSheet(
        val customListId: CustomListId,
        val customListName: CustomListName,
        val item: RelayItem.Location,
    ) : LocationBottomSheetState

    data class ShowLocationBottomSheet(
        val customLists: List<RelayItem.CustomList>,
        val item: RelayItem.Location,
    ) : LocationBottomSheetState

    data class ShowEditCustomListBottomSheet(val customList: RelayItem.CustomList) :
        LocationBottomSheetState
}

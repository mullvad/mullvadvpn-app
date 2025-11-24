package net.mullvad.mullvadvpn.compose.screen.location

import android.content.Context
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideIn
import androidx.compose.animation.slideOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Column
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SheetState
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.IntOffset
import androidx.lifecycle.compose.collectAsStateWithLifecycle
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
import net.mullvad.mullvadvpn.compose.state.LocationBottomSheetUiState
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.relaylist.canAddLocation
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.location.LocationBottomSheetViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Composable
internal fun LocationBottomSheets(
    locationBottomSheetState: LocationBottomSheetState?,
    onCreateCustomList: (RelayItem.Location?) -> Unit,
    onAddLocationToList: (RelayItem.Location, RelayItem.CustomList) -> Unit,
    onRemoveLocationFromList: (location: RelayItem.Location, parent: CustomListId) -> Unit,
    onEditCustomListName: (RelayItem.CustomList) -> Unit,
    onEditLocationsCustomList: (RelayItem.CustomList) -> Unit,
    onDeleteCustomList: (RelayItem.CustomList) -> Unit,
    onSetAsEntry: (RelayItem) -> Unit,
    onDisableMultihop: () -> Unit,
    onSetAsExit: (RelayItem) -> Unit,
    onHideBottomSheet: () -> Unit,
) {
    if (locationBottomSheetState != null) {
        val viewModel =
            koinViewModel<LocationBottomSheetViewModel>(
                key = locationBottomSheetState.toString(),
                parameters = { parametersOf(locationBottomSheetState) },
            )

        val state by viewModel.uiState.collectAsStateWithLifecycle()
        LocationBottomSheets(
            locationBottomSheetUiState = state,
            onCreateCustomList = onCreateCustomList,
            onAddLocationToList = onAddLocationToList,
            onRemoveLocationFromList = onRemoveLocationFromList,
            onEditCustomListName = onEditCustomListName,
            onEditLocationsCustomList = onEditLocationsCustomList,
            onDeleteCustomList = onDeleteCustomList,
            onSetAsEntry = onSetAsEntry,
            onDisableMultihop = onDisableMultihop,
            onSetAsExit = onSetAsExit,
            onHideBottomSheet = onHideBottomSheet,
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun LocationBottomSheets(
    locationBottomSheetUiState: Lc<Unit, LocationBottomSheetUiState>,
    onCreateCustomList: (RelayItem.Location?) -> Unit,
    onAddLocationToList: (RelayItem.Location, RelayItem.CustomList) -> Unit,
    onRemoveLocationFromList: (location: RelayItem.Location, parent: CustomListId) -> Unit,
    onEditCustomListName: (RelayItem.CustomList) -> Unit,
    onEditLocationsCustomList: (RelayItem.CustomList) -> Unit,
    onDeleteCustomList: (RelayItem.CustomList) -> Unit,
    onSetAsEntry: (RelayItem) -> Unit,
    onDisableMultihop: () -> Unit,
    onSetAsExit: (RelayItem) -> Unit,
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

    when (val state = locationBottomSheetUiState.contentOrNull()) {
        is LocationBottomSheetUiState.Location -> {
            LocationBottomSheet(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                sheetState = sheetState,
                customLists = state.customLists,
                item = state.item,
                canBeSetAsEntry = state.canBeSetAsEntry,
                canBeSetAsExit = state.canBeSetAsExit,
                canBeRemovedAsEntry = state.canBeRemovedAsEntry,
                onCreateCustomList = onCreateCustomList,
                onAddLocationToList = onAddLocationToList,
                onSetAsEntry = onSetAsEntry,
                onDisableMultihop = onDisableMultihop,
                onSetAsExit = onSetAsExit,
                closeBottomSheet = onCloseBottomSheet,
            )
        }
        is LocationBottomSheetUiState.CustomList -> {
            EditCustomListBottomSheet(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                sheetState = sheetState,
                customList = state.item,
                canBeSetAsEntry = state.canBeSetAsEntry,
                canBeSetAsExit = state.canBeSetAsExit,
                canBeRemovedAsEntry = state.canBeRemovedAsEntry,
                onEditName = onEditCustomListName,
                onEditLocations = onEditLocationsCustomList,
                onDeleteCustomList = onDeleteCustomList,
                onSetAsEntry = onSetAsEntry,
                onDisableMultihop = onDisableMultihop,
                onSetAsExit = onSetAsExit,
                closeBottomSheet = onCloseBottomSheet,
            )
        }
        is LocationBottomSheetUiState.CustomListsEntry -> {
            CustomListEntryBottomSheet(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                sheetState = sheetState,
                customListId = state.customListId,
                customListName = state.customListName,
                item = state.item,
                canBeSetAsEntry = state.canBeSetAsEntry,
                canBeSetAsExit = state.canBeSetAsExit,
                canBeRemovedAsEntry = state.canBeRemovedAsEntry,
                onRemoveLocationFromList = onRemoveLocationFromList,
                onSetAsEntry = onSetAsEntry,
                onDisableMultihop = onDisableMultihop,
                onSetAsExit = onSetAsExit,
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
    canBeSetAsEntry: Boolean,
    canBeSetAsExit: Boolean,
    canBeRemovedAsEntry: Boolean,
    onCreateCustomList: (relayItem: RelayItem.Location) -> Unit,
    onAddLocationToList: (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit,
    onDisableMultihop: () -> Unit,
    onSetAsEntry: (RelayItem.Location) -> Unit,
    onSetAsExit: (RelayItem.Location) -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit,
) {
    MullvadModalBottomSheet(
        sheetState = sheetState,
        backgroundColor = backgroundColor,
        onBackgroundColor = onBackgroundColor,
        onDismissRequest = { closeBottomSheet(false) },
        modifier = Modifier.testTag(SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG),
    ) { ->
        var showAddToListState by remember { mutableStateOf(false) }
        HeaderCell(
            text =
                if (showAddToListState) {
                    stringResource(id = R.string.add_location_to_list, item.name)
                } else {
                    item.name
                },
            background = backgroundColor,
        )
        HorizontalDivider(color = onBackgroundColor)
        AnimatedContent(
            targetState = showAddToListState to customLists,
            transitionSpec = {
                slideIn { IntOffset(it.width, 0) } + fadeIn() togetherWith
                    slideOut { IntOffset(-it.width, 0) } + fadeOut()
            },
            label = "Show add to list",
        ) { (showAddToList, customLists) ->
            if (showAddToList) {
                CustomLists(
                    customLists = customLists,
                    item = item,
                    onBackgroundColor = onBackgroundColor,
                    onAddLocationToList = onAddLocationToList,
                    onCreateCustomList = onCreateCustomList,
                    closeBottomSheet = closeBottomSheet,
                )
            } else {
                Column {
                    IconCell(
                        imageVector = null,
                        title = stringResource(id = R.string.add_to_list),
                        titleColor = onBackgroundColor,
                        onClick = { showAddToListState = true },
                        endIcon = {
                            Icon(
                                imageVector = Icons.Default.ChevronRight,
                                contentDescription = null,
                                tint = onBackgroundColor,
                            )
                        },
                    )
                    MultihopOptions(
                        item = item,
                        canBeSetAsEntry = canBeSetAsEntry,
                        canBeSetAsExit = canBeSetAsExit,
                        canBeRemovedAsEntry = canBeRemovedAsEntry,
                        onBackgroundColor = onBackgroundColor,
                        onSetAsEntry = onSetAsEntry,
                        onSetAsExit = onSetAsExit,
                        onDisableMultihop = onDisableMultihop,
                        closeBottomSheet = closeBottomSheet,
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun EditCustomListBottomSheet(
    backgroundColor: Color,
    onBackgroundColor: Color,
    sheetState: SheetState,
    customList: RelayItem.CustomList,
    canBeSetAsEntry: Boolean,
    canBeSetAsExit: Boolean,
    canBeRemovedAsEntry: Boolean,
    onEditName: (item: RelayItem.CustomList) -> Unit,
    onEditLocations: (item: RelayItem.CustomList) -> Unit,
    onDeleteCustomList: (item: RelayItem.CustomList) -> Unit,
    onSetAsEntry: (RelayItem.CustomList) -> Unit,
    onDisableMultihop: () -> Unit,
    onSetAsExit: (RelayItem.CustomList) -> Unit,
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
        MultihopOptions(
            item = customList,
            canBeSetAsEntry = canBeSetAsEntry,
            canBeSetAsExit = canBeSetAsExit,
            canBeRemovedAsEntry = canBeRemovedAsEntry,
            onBackgroundColor = onBackgroundColor,
            onSetAsEntry = onSetAsEntry,
            onSetAsExit = onSetAsExit,
            onDisableMultihop = onDisableMultihop,
            closeBottomSheet = closeBottomSheet,
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
    canBeSetAsEntry: Boolean,
    canBeSetAsExit: Boolean,
    canBeRemovedAsEntry: Boolean,
    onRemoveLocationFromList: (location: RelayItem.Location, customListId: CustomListId) -> Unit,
    onSetAsEntry: (RelayItem.Location) -> Unit,
    onDisableMultihop: () -> Unit,
    onSetAsExit: (RelayItem.Location) -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit,
) {
    MullvadModalBottomSheet(
        sheetState = sheetState,
        backgroundColor = backgroundColor,
        onBackgroundColor = onBackgroundColor,
        onDismissRequest = { closeBottomSheet(false) },
        modifier = Modifier.testTag(SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG),
    ) {
        HeaderCell(text = item.name, background = backgroundColor)
        HorizontalDivider(color = onBackgroundColor)

        IconCell(
            imageVector = Icons.Default.Remove,
            title =
                stringResource(id = R.string.remove_location_from_list, item.name, customListName),
            titleColor = onBackgroundColor,
            onClick = {
                onRemoveLocationFromList(item, customListId)
                closeBottomSheet(true)
            },
        )
        MultihopOptions(
            item = item,
            canBeSetAsEntry = canBeSetAsEntry,
            canBeSetAsExit = canBeSetAsExit,
            canBeRemovedAsEntry = canBeRemovedAsEntry,
            onBackgroundColor = onBackgroundColor,
            onSetAsEntry = onSetAsEntry,
            onSetAsExit = onSetAsExit,
            onDisableMultihop = onDisableMultihop,
            closeBottomSheet = closeBottomSheet,
        )
    }
}

@Composable
private fun CustomLists(
    customLists: List<RelayItem.CustomList>,
    item: RelayItem.Location,
    onBackgroundColor: Color,
    onAddLocationToList: (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit,
    onCreateCustomList: (location: RelayItem.Location) -> Unit,
    closeBottomSheet: (Boolean) -> Unit,
) {
    Column {
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

@Composable
private fun <T : RelayItem> MultihopOptions(
    item: T,
    canBeSetAsEntry: Boolean,
    canBeSetAsExit: Boolean,
    canBeRemovedAsEntry: Boolean,
    onBackgroundColor: Color,
    onSetAsEntry: (T) -> Unit,
    onSetAsExit: (T) -> Unit,
    onDisableMultihop: () -> Unit,
    closeBottomSheet: (Boolean) -> Unit,
) {
    if (canBeRemovedAsEntry) {
        IconCell(
            imageVector = null,
            title = stringResource(R.string.remove_as_multihop_entry),
            titleColor = onBackgroundColor,
            onClick = {
                onDisableMultihop()
                closeBottomSheet(true)
            },
        )
    } else {
        IconCell(
            imageVector = null,
            title =
                stringResource(
                    if (canBeSetAsEntry) {
                        R.string.set_as_multihop_entry
                    } else {
                        R.string.set_as_multihop_entry_unavailable
                    }
                ),
            titleColor =
                if (canBeSetAsEntry) {
                    onBackgroundColor
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant
                },
            onClick = {
                onSetAsEntry(item)
                closeBottomSheet(true)
            },
            enabled = canBeSetAsEntry,
        )
    }
    IconCell(
        imageVector = null,
        title =
            stringResource(
                if (canBeSetAsExit) {
                    R.string.set_as_multihop_exit
                } else {
                    R.string.set_as_multihop_exit_unavailable
                }
            ),
        titleColor =
            if (canBeSetAsExit) {
                onBackgroundColor
            } else {
                MaterialTheme.colorScheme.onSurfaceVariant
            },
        onClick = {
            onSetAsExit(item)
            closeBottomSheet(true)
        },
        enabled = canBeSetAsExit,
    )
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
    val item: RelayItem

    data class ShowCustomListsEntryBottomSheet(
        val customListId: CustomListId,
        override val item: RelayItem.Location,
    ) : LocationBottomSheetState

    data class ShowLocationBottomSheet(override val item: RelayItem.Location) :
        LocationBottomSheetState

    data class ShowEditCustomListBottomSheet(override val item: RelayItem.CustomList) :
        LocationBottomSheetState
}

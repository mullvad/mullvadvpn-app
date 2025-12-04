package net.mullvad.mullvadvpn.compose.screen.location

import android.content.res.Resources
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material.icons.outlined.AddLocationAlt
import androidx.compose.material.icons.outlined.Dns
import androidx.compose.material.icons.outlined.LocationOn
import androidx.compose.material.icons.outlined.WrongLocation
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SheetState
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import com.ramcosta.composedestinations.spec.DestinationSpec
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.component.MullvadModalBottomSheet
import net.mullvad.mullvadvpn.compose.state.LocationBottomSheetUiState
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SetAsState
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.designsystem.RelayListHeaderTokens
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.relaylist.canAddLocation
import net.mullvad.mullvadvpn.usecase.ModifyMultihopError
import net.mullvad.mullvadvpn.usecase.MultihopChange
import net.mullvad.mullvadvpn.usecase.SelectRelayItemError
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.location.LocationBottomSheetViewModel
import net.mullvad.mullvadvpn.viewmodel.location.UndoChangeMultihopAction
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
    onModifyMultihopError: (ModifyMultihopError, MultihopChange) -> Unit,
    onRelayItemError: (SelectRelayItemError) -> Unit,
    onMultihopChanged: (UndoChangeMultihopAction) -> Unit,
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
            onSetAsEntry = {
                viewModel.setAsEntry(item = it, onError = onModifyMultihopError, onMultihopChanged)
            },
            onDisableMultihop = { viewModel.disableMultihop(onMultihopChanged) },
            onSetAsExit = {
                viewModel.setAsExit(
                    item = it,
                    onModifyMultihopError = onModifyMultihopError,
                    onRelayItemError = onRelayItemError,
                    onMultihopChanged,
                )
            },
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
                setAsEntryState = state.setAsEntryState,
                setAsExitState = state.setAsExitState,
                canBeRemovedAsEntry = state.canDisableMultihop,
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
                setAsEntryState = state.setAsEntryState,
                setAsExitState = state.setAsExitState,
                canBeRemovedAsEntry = state.canDisableMultihop,
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
                setAsEntryState = state.setAsEntryState,
                setAsExitState = state.setAsExitState,
                canBeRemovedAsEntry = state.canDisableMultihop,
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
    setAsEntryState: SetAsState,
    setAsExitState: SetAsState,
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
    ) { bottomPadding ->
        val scrollState = rememberScrollState()
        Column(modifier = Modifier.verticalScroll(scrollState).padding(bottom = bottomPadding)) {
            Header(text = item.name, color = onBackgroundColor)
            HorizontalDivider(color = onBackgroundColor)
            MultihopOptions(
                item = item,
                setAsEntryState = setAsEntryState,
                setAsExitState = setAsExitState,
                canBeRemovedAsEntry = canBeRemovedAsEntry,
                onBackgroundColor = onBackgroundColor,
                onSetAsEntry = onSetAsEntry,
                onSetAsExit = onSetAsExit,
                onDisableMultihop = onDisableMultihop,
                closeBottomSheet = closeBottomSheet,
            )
            SubHeader(
                text = stringResource(R.string.add_to_list),
                onBackgroundColor = onBackgroundColor,
            )
            CustomLists(
                customLists = customLists,
                item = item,
                onBackgroundColor = onBackgroundColor,
                onAddLocationToList = onAddLocationToList,
                onCreateCustomList = onCreateCustomList,
                closeBottomSheet = closeBottomSheet,
            )
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
    setAsEntryState: SetAsState,
    setAsExitState: SetAsState,
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
        Header(text = customList.name, color = onBackgroundColor)
        HorizontalDivider(color = onBackgroundColor)
        MultihopOptions(
            item = customList,
            setAsEntryState = setAsEntryState,
            setAsExitState = setAsExitState,
            canBeRemovedAsEntry = canBeRemovedAsEntry,
            onBackgroundColor = onBackgroundColor,
            onSetAsEntry = onSetAsEntry,
            onSetAsExit = onSetAsExit,
            onDisableMultihop = onDisableMultihop,
            closeBottomSheet = closeBottomSheet,
        )
        SubHeader(text = stringResource(R.string.edit_list), onBackgroundColor = onBackgroundColor)
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
    setAsEntryState: SetAsState,
    setAsExitState: SetAsState,
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
        Header(text = item.name, color = onBackgroundColor)
        HorizontalDivider(color = onBackgroundColor)
        MultihopOptions(
            item = item,
            setAsEntryState = setAsEntryState,
            setAsExitState = setAsExitState,
            canBeRemovedAsEntry = canBeRemovedAsEntry,
            onBackgroundColor = onBackgroundColor,
            onSetAsEntry = onSetAsEntry,
            onSetAsExit = onSetAsExit,
            onDisableMultihop = onDisableMultihop,
            closeBottomSheet = closeBottomSheet,
        )
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

@Composable
private fun SubHeader(text: String, onBackgroundColor: Color) {
    Row(
        modifier =
            Modifier.defaultMinSize(minHeight = SUB_HEADER_HEADER_MIN_HEIGHT)
                .height(IntrinsicSize.Min)
                .padding(horizontal = Dimens.mediumPadding),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(text = text, style = MaterialTheme.typography.labelLarge, color = onBackgroundColor)
        HorizontalDivider(
            Modifier.weight(1f, true).padding(start = Dimens.smallPadding),
            color =
                MaterialTheme.colorScheme.onBackground.copy(
                    alpha = RelayListHeaderTokens.RelayListHeaderDividerAlpha
                ),
        )
    }
}

@Composable
private fun Header(text: String, color: Color) {
    Text(
        text = text,
        color = color,
        style = MaterialTheme.typography.headlineSmall.merge(fontWeight = FontWeight.Normal),
        modifier =
            Modifier.padding(horizontal = Dimens.mediumPadding, vertical = Dimens.smallPadding),
    )
}

@Composable
private fun <T : RelayItem> MultihopOptions(
    item: T,
    setAsEntryState: SetAsState,
    setAsExitState: SetAsState,
    canBeRemovedAsEntry: Boolean,
    onBackgroundColor: Color,
    onSetAsEntry: (T) -> Unit,
    onSetAsExit: (T) -> Unit,
    onDisableMultihop: () -> Unit,
    closeBottomSheet: (Boolean) -> Unit,
) {
    if (setAsExitState != SetAsState.HIDDEN) {
        val enabled = setAsExitState == SetAsState.ENABLED
        IconCell(
            imageVector =
                if (setAsEntryState == SetAsState.HIDDEN) {
                    Icons.Outlined.AddLocationAlt
                } else {
                    Icons.Outlined.LocationOn
                },
            title =
                stringResource(
                    if (enabled) {
                        R.string.set_as_multihop_exit
                    } else {
                        R.string.set_as_multihop_exit_unavailable
                    }
                ),
            titleColor =
                if (enabled) {
                    onBackgroundColor
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant
                },
            onClick = {
                onSetAsExit(item)
                closeBottomSheet(true)
            },
            enabled = enabled,
        )
    }
    if (canBeRemovedAsEntry) {
        IconCell(
            imageVector = Icons.Outlined.WrongLocation,
            title = stringResource(R.string.disable_multihop),
            titleColor = onBackgroundColor,
            onClick = {
                onDisableMultihop()
                closeBottomSheet(true)
            },
        )
    } else if (setAsEntryState != SetAsState.HIDDEN) {
        val enabled = setAsEntryState == SetAsState.ENABLED
        IconCell(
            imageVector =
                if (setAsExitState == SetAsState.HIDDEN) {
                    Icons.Outlined.AddLocationAlt
                } else {
                    Icons.Outlined.Dns
                },
            title =
                stringResource(
                    if (enabled) {
                        R.string.set_as_multihop_entry
                    } else {
                        R.string.set_as_multihop_entry_unavailable
                    }
                ),
            titleColor =
                if (enabled) {
                    onBackgroundColor
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant
                },
            onClick = {
                onSetAsEntry(item)
                closeBottomSheet(true)
            },
            enabled = enabled,
        )
    }
}

internal suspend fun SnackbarHostState.showResultSnackbar(
    resources: Resources,
    result: CustomListActionResultData,
    onUndo: (CustomListAction) -> Unit,
) {

    showSnackbarImmediately(
        message = result.message(resources),
        actionLabel =
            if (result is CustomListActionResultData.Success) resources.getString(R.string.undo)
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

private fun CustomListActionResultData.message(resources: Resources): String =
    when (this) {
        is CustomListActionResultData.Success.CreatedWithLocations ->
            if (locationNames.size == 1) {
                resources.getString(
                    R.string.location_was_added_to_list,
                    locationNames.first(),
                    customListName,
                )
            } else {
                resources.getString(R.string.create_custom_list_message, customListName)
            }
        is CustomListActionResultData.Success.Deleted ->
            resources.getString(R.string.delete_custom_list_message, customListName)
        is CustomListActionResultData.Success.LocationAdded ->
            resources.getString(R.string.location_was_added_to_list, locationName, customListName)
        is CustomListActionResultData.Success.LocationRemoved ->
            resources.getString(
                R.string.location_was_removed_from_list,
                locationName,
                customListName,
            )
        is CustomListActionResultData.Success.LocationChanged ->
            resources.getString(R.string.locations_were_changed_for, customListName)
        is CustomListActionResultData.Success.Renamed ->
            resources.getString(R.string.name_was_changed_to, newName)
        CustomListActionResultData.GenericError -> resources.getString(R.string.error_occurred)
    }

@Composable
internal fun <D : DestinationSpec, R : CustomListActionResultData> ResultRecipient<D, R>
    .OnCustomListNavResult(
    snackbarHostState: SnackbarHostState,
    performAction: (action: CustomListAction) -> Unit,
) {
    val scope = rememberCoroutineScope()
    val resources = LocalResources.current
    this.onNavResult { result ->
        when (result) {
            NavResult.Canceled -> {
                /* Do nothing */
            }
            is NavResult.Value -> {
                // Handle result
                scope.launch {
                    snackbarHostState.showResultSnackbar(
                        resources = resources,
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
    val relayListType: RelayListType

    data class ShowCustomListsEntryBottomSheet(
        val customListId: CustomListId,
        override val item: RelayItem.Location,
        override val relayListType: RelayListType,
    ) : LocationBottomSheetState

    data class ShowLocationBottomSheet(
        override val item: RelayItem.Location,
        override val relayListType: RelayListType,
    ) : LocationBottomSheetState

    data class ShowEditCustomListBottomSheet(
        override val item: RelayItem.CustomList,
        override val relayListType: RelayListType,
    ) : LocationBottomSheetState
}

private val SUB_HEADER_HEADER_MIN_HEIGHT = 48.dp

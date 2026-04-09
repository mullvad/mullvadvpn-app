package net.mullvad.mullvadvpn.feature.location.impl.bottomsheet

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
import androidx.compose.material.icons.outlined.AddLocationAlt
import androidx.compose.material.icons.outlined.Dns
import androidx.compose.material.icons.outlined.LocationOn
import androidx.compose.material.icons.outlined.WrongLocation
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material.icons.rounded.Remove
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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.goBack
import net.mullvad.mullvadvpn.common.compose.navigateReplaceTop
import net.mullvad.mullvadvpn.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customlist.api.CreateCustomListNavKey
import net.mullvad.mullvadvpn.feature.customlist.api.DeleteCustomListNavKey
import net.mullvad.mullvadvpn.feature.customlist.api.EditCustomListLocationsNavKey
import net.mullvad.mullvadvpn.feature.customlist.api.EditCustomListNameNavKey
import net.mullvad.mullvadvpn.feature.location.api.LocationBottomSheetState
import net.mullvad.mullvadvpn.feature.location.impl.R
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.util.relaylist.canAddLocation
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.communication.CustomListAction
import net.mullvad.mullvadvpn.lib.model.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.lib.ui.component.MullvadModalBottomSheet
import net.mullvad.mullvadvpn.lib.ui.component.listitem.BottomSheetListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.IconListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.designsystem.RelayListHeaderTokens
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_CUSTOM_LIST_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SELECT_LOCATION_LOCATION_BOTTOM_SHEET_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@OptIn(ExperimentalMaterial3Api::class)
@Suppress("LongMethod")
@Composable
internal fun LocationBottomSheets(
    navigator: Navigator,
    locationBottomSheetState: LocationBottomSheetState,
) {
    val vm =
        koinViewModel<LocationBottomSheetViewModel>(
            key = locationBottomSheetState.toString(),
            parameters = { parametersOf(locationBottomSheetState) },
        )

    val resultStore = LocalResultStore.current

    CollectSideEffectWithLifecycle(vm.uiSideEffect) { resultStore.setResult(it) }

    val state by vm.uiState.collectAsStateWithLifecycle()
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()

    LocationBottomSheets(
        locationBottomSheetUiState = state,
        sheetState = sheetState,
        onCreateCustomList =
            dropUnlessResumed { relayItem ->
                navigator.navigateReplaceTop(
                    sheetState,
                    scope,
                    CreateCustomListNavKey(locationCode = relayItem?.id)
                )
            },
        onAddLocationToList = vm::addLocationToList,
        onRemoveLocationFromList = vm::removeLocationFromList,
        onEditCustomListName =
            dropUnlessResumed { customList: RelayItem.CustomList ->
                navigator.navigateReplaceTop(
                    sheetState, scope,
                    EditCustomListNameNavKey(
                        customListId = customList.id,
                        initialName = customList.customList.name,
                    )
                )
            },
        onEditLocationsCustomList =
            dropUnlessResumed { customList: RelayItem.CustomList ->
                navigator.navigateReplaceTop(
                    sheetState,
                    scope,
                    EditCustomListLocationsNavKey(customListId = customList.id, newList = false)
                )
            },
        onDeleteCustomList =
            dropUnlessResumed { customList: RelayItem.CustomList ->
                navigator.navigateReplaceTop(
                    sheetState, scope,
                    DeleteCustomListNavKey(
                        customListId = customList.id,
                        name = customList.customList.name,
                    )
                )

            },
        onSetAsEntry = {
            vm.setAsEntry(
                item = it,
                onError = vm::onModifyMultihopError,
                onUpdateMultihop = vm::onMultihopChanged,
            )
        },
        onDisableMultihop = { vm.disableMultihop(vm::onMultihopChanged) },
        onSetAsExit = {
            vm.setAsExit(
                item = it,
                onModifyMultihopError = vm::onModifyMultihopError,
                onRelayItemError = vm::onSelectRelayItemError,
                onUpdateMultihop = vm::onMultihopChanged,
            )
        },
        closeBottomSheet = { animate ->
            if (animate) {
                navigator.goBack(sheetState, scope)
            } else {
                navigator.goBack()
            }
        },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun LocationBottomSheets(
    locationBottomSheetUiState: Lc<Unit, LocationBottomSheetUiState>,
    sheetState: SheetState,
    onCreateCustomList: (RelayItem.Location?) -> Unit,
    onAddLocationToList: (RelayItem.Location, RelayItem.CustomList) -> Unit,
    onRemoveLocationFromList: (location: RelayItem.Location, parent: CustomListId) -> Unit,
    onEditCustomListName: (RelayItem.CustomList) -> Unit,
    onEditLocationsCustomList: (RelayItem.CustomList) -> Unit,
    onDeleteCustomList: (RelayItem.CustomList) -> Unit,
    onSetAsEntry: (RelayItem) -> Unit,
    onDisableMultihop: () -> Unit,
    onSetAsExit: (RelayItem) -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit,
) {
    val backgroundColor: Color = MaterialTheme.colorScheme.secondaryContainer
    val onBackgroundColor: Color = MaterialTheme.colorScheme.onSecondary

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
                closeBottomSheet = closeBottomSheet,
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
                closeBottomSheet = closeBottomSheet,
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
                closeBottomSheet = closeBottomSheet,
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
                backgroundColor = backgroundColor,
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
                backgroundColor = backgroundColor,
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
            backgroundColor = backgroundColor,
            onBackgroundColor = onBackgroundColor,
            onSetAsEntry = onSetAsEntry,
            onSetAsExit = onSetAsExit,
            onDisableMultihop = onDisableMultihop,
            closeBottomSheet = closeBottomSheet,
        )
        SubHeader(text = stringResource(R.string.edit_list), onBackgroundColor = onBackgroundColor)
        IconListItem(
            leadingIcon = Icons.Rounded.Edit,
            title = stringResource(id = R.string.edit_name),
            colors =
                ListItemDefaults.colors(
                    headlineColor = onBackgroundColor,
                    containerColorParent = backgroundColor,
                ),
            position = Position.Middle,
            onClick = { onEditName(customList) },
        )
        IconListItem(
            leadingIcon = Icons.Rounded.Add,
            title = stringResource(id = R.string.edit_locations),
            colors =
                ListItemDefaults.colors(
                    headlineColor = onBackgroundColor,
                    containerColorParent = backgroundColor,
                ),
            position = Position.Middle,
            onClick = { onEditLocations(customList) },
        )
        IconListItem(
            leadingIcon = Icons.Rounded.Delete,
            title = stringResource(id = R.string.delete),
            colors =
                ListItemDefaults.colors(
                    headlineColor = onBackgroundColor,
                    containerColorParent = backgroundColor,
                ),
            position = Position.Middle,
            onClick = { onDeleteCustomList(customList) },
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
            backgroundColor = backgroundColor,
            onBackgroundColor = onBackgroundColor,
            onSetAsEntry = onSetAsEntry,
            onSetAsExit = onSetAsExit,
            onDisableMultihop = onDisableMultihop,
            closeBottomSheet = closeBottomSheet,
        )
        IconListItem(
            leadingIcon = Icons.Rounded.Remove,
            title =
                stringResource(id = R.string.remove_location_from_list, item.name, customListName),
            colors =
                ListItemDefaults.colors(
                    headlineColor = onBackgroundColor,
                    containerColorParent = backgroundColor,
                ),
            position = Position.Middle,
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
    backgroundColor: Color,
    onBackgroundColor: Color,
    onAddLocationToList: (location: RelayItem.Location, customList: RelayItem.CustomList) -> Unit,
    onCreateCustomList: (location: RelayItem.Location) -> Unit,
    closeBottomSheet: (Boolean) -> Unit,
) {
    customLists.forEach {
        val enabled = it.canAddLocation(item)
        BottomSheetListItem(
            title =
                if (enabled) {
                    it.name
                } else {
                    stringResource(id = R.string.location_added, it.name)
                },
            backgroundColor = backgroundColor,
            onBackgroundColor = onBackgroundColor,
            onClick = {
                onAddLocationToList(item, it)
                closeBottomSheet(true)
            },
            isEnabled = enabled,
        )
    }
    IconListItem(
        leadingIcon = Icons.Rounded.Add,
        title = stringResource(id = R.string.new_list),
        colors =
            ListItemDefaults.colors(
                headlineColor = onBackgroundColor,
                containerColorParent = backgroundColor,
            ),
        position = Position.Middle,
        onClick = { onCreateCustomList(item) },
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
    backgroundColor: Color,
    onBackgroundColor: Color,
    onSetAsEntry: (T) -> Unit,
    onSetAsExit: (T) -> Unit,
    onDisableMultihop: () -> Unit,
    closeBottomSheet: (Boolean) -> Unit,
) {
    if (setAsExitState != SetAsState.HIDDEN) {
        val enabled = setAsExitState == SetAsState.ENABLED
        IconListItem(
            leadingIcon =
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
            colors =
                ListItemDefaults.colors(
                    headlineColor = onBackgroundColor,
                    containerColorParent = backgroundColor,
                ),
            position = Position.Middle,
            onClick = {
                onSetAsExit(item)
                closeBottomSheet(true)
            },
            isEnabled = enabled,
        )
    }
    if (canBeRemovedAsEntry) {
        IconListItem(
            leadingIcon = Icons.Outlined.WrongLocation,
            title = stringResource(R.string.disable_multihop),
            colors =
                ListItemDefaults.colors(
                    headlineColor = onBackgroundColor,
                    containerColorParent = backgroundColor,
                ),
            position = Position.Middle,
            onClick = {
                onDisableMultihop()
                closeBottomSheet(true)
            },
        )
    } else if (setAsEntryState != SetAsState.HIDDEN) {
        val enabled = setAsEntryState == SetAsState.ENABLED
        IconListItem(
            leadingIcon =
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
            colors =
                ListItemDefaults.colors(
                    headlineColor = onBackgroundColor,
                    containerColorParent = backgroundColor,
                ),
            position = Position.Middle,
            onClick = {
                onSetAsEntry(item)
                closeBottomSheet(true)
            },
            isEnabled = enabled,
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

private val SUB_HEADER_HEADER_MIN_HEIGHT = 48.dp

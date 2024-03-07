package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.TwoRowCell
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.compose.destinations.CustomListLocationsDestination
import net.mullvad.mullvadvpn.compose.destinations.DeleteCustomListDestination
import net.mullvad.mullvadvpn.compose.destinations.EditCustomListNameDestination
import net.mullvad.mullvadvpn.compose.state.EditCustomListState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.DELETE_DROPDOWN_MENU_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.TOP_BAR_DROPDOWN_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.viewmodel.EditCustomListViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewEditCustomListScreen() {
    AppTheme {
        EditCustomListScreen(
            state =
                EditCustomListState.Content(
                    id = "id",
                    name = "Custom list",
                    locations =
                        listOf(
                            RelayItem.Relay(
                                "Relay",
                                "Relay",
                                true,
                                GeographicLocationConstraint.Hostname(
                                    "hostname",
                                    "hostname",
                                    "hostname"
                                )
                            )
                        )
                )
        )
    }
}

@Composable
@Destination(style = SlideInFromRightTransition::class)
fun EditCustomList(
    navigator: DestinationsNavigator,
    backNavigator: ResultBackNavigator<CustomListResult.Deleted>,
    customListId: String,
    confirmDeleteListResultRecipient:
        ResultRecipient<DeleteCustomListDestination, CustomListResult.Deleted>
) {
    val viewModel =
        koinViewModel<EditCustomListViewModel>(parameters = { parametersOf(customListId) })

    confirmDeleteListResultRecipient.onNavResult {
        when (it) {
            NavResult.Canceled -> {
                // Do nothing
            }
            is NavResult.Value -> backNavigator.navigateBack(result = it.value)
        }
    }

    val state by viewModel.uiState.collectAsStateWithLifecycle()
    EditCustomListScreen(
        state = state,
        onDeleteList = { name ->
            navigator.navigate(
                DeleteCustomListDestination(customListId = customListId, name = name)
            ) {
                launchSingleTop = true
            }
        },
        onNameClicked = { id, name ->
            navigator.navigate(
                EditCustomListNameDestination(customListId = id, initialName = name)
            ) {
                launchSingleTop = true
            }
        },
        onLocationsClicked = {
            navigator.navigate(CustomListLocationsDestination(customListId = it, newList = false)) {
                launchSingleTop = true
            }
        },
        onBackClick = backNavigator::navigateBack
    )
}

@Composable
fun EditCustomListScreen(
    state: EditCustomListState,
    onDeleteList: (name: String) -> Unit = {},
    onNameClicked: (id: String, name: String) -> Unit = { _, _ -> },
    onLocationsClicked: (String) -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    val title =
        when (state) {
            EditCustomListState.Loading,
            EditCustomListState.NotFound -> ""
            is EditCustomListState.Content -> state.name
        }
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.edit_list),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        actions = { Actions(onDeleteList = { onDeleteList(title) }) },
    ) { modifier: Modifier ->
        SpacedColumn(modifier = modifier, alignment = Alignment.Top) {
            when (state) {
                EditCustomListState.Loading -> {
                    MullvadCircularProgressIndicatorLarge(
                        modifier = Modifier.testTag(CIRCULAR_PROGRESS_INDICATOR)
                    )
                }
                EditCustomListState.NotFound -> {
                    Text(
                        text = stringResource(id = R.string.not_found),
                        modifier = Modifier.padding(Dimens.screenVerticalMargin),
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSecondary
                    )
                }
                is EditCustomListState.Content -> {
                    // Name cell
                    TwoRowCell(
                        titleText = stringResource(id = R.string.list_name),
                        subtitleText = state.name,
                        onCellClicked = { onNameClicked(state.id, state.name) }
                    )
                    // Locations cell
                    TwoRowCell(
                        titleText = stringResource(id = R.string.locations),
                        subtitleText =
                            pluralStringResource(
                                id = R.plurals.number_of_locations,
                                state.locations.size,
                                state.locations.size
                            ),
                        onCellClicked = { onLocationsClicked(state.id) }
                    )
                }
            }
        }
    }
}

@Composable
private fun Actions(onDeleteList: () -> Unit) {
    var showMenu by remember { mutableStateOf(false) }
    IconButton(
        onClick = { showMenu = true },
        modifier = Modifier.testTag(TOP_BAR_DROPDOWN_BUTTON_TEST_TAG)
    ) {
        Icon(painter = painterResource(id = R.drawable.icon_more_vert), contentDescription = null)
        if (showMenu) {
            DropdownMenu(
                expanded = true,
                onDismissRequest = { showMenu = false },
                modifier = Modifier.background(MaterialTheme.colorScheme.surfaceContainer)
            ) {
                DropdownMenuItem(
                    text = { Text(text = stringResource(id = R.string.delete_list)) },
                    leadingIcon = {
                        Icon(
                            painter = painterResource(id = R.drawable.icon_delete),
                            contentDescription = null,
                        )
                    },
                    colors =
                        MenuDefaults.itemColors()
                            .copy(
                                leadingIconColor = MaterialTheme.colorScheme.onSurface,
                                textColor = MaterialTheme.colorScheme.onSurface,
                            ),
                    onClick = {
                        onDeleteList()
                        showMenu = false
                    },
                    modifier = Modifier.testTag(DELETE_DROPDOWN_MENU_ITEM_TEST_TAG)
                )
            }
        }
    }
}

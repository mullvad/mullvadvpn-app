package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.TwoRowCell
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.compose.destinations.CustomListLocationsDestination
import net.mullvad.mullvadvpn.compose.destinations.DeleteCustomListConfirmationDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.EditCustomListNameDestination
import net.mullvad.mullvadvpn.compose.state.EditCustomListState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.EditCustomListSideEffect
import net.mullvad.mullvadvpn.viewmodel.EditCustomListViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
fun PreviewEditCustomListScreen() {
    AppTheme {
        EditCustomListScreen(
            uiState =
                EditCustomListState.Content(id = "id", name = "Custom list", numberOfLocations = 3)
        )
    }
}

@Composable
@Destination(style = SlideInFromRightTransition::class)
fun EditCustomList(
    navigator: DestinationsNavigator,
    customListId: String,
    confirmDeleteListResultRecipient:
        ResultRecipient<DeleteCustomListConfirmationDialogDestination, Boolean>
) {
    val viewModel =
        koinViewModel<EditCustomListViewModel>(parameters = { parametersOf(customListId) })

    LaunchedEffect(Unit) {
        viewModel.uiSideEffect.collect {
            when (it) {
                is EditCustomListSideEffect.CloseScreen -> navigator.navigateUp()
            }
        }
    }

    confirmDeleteListResultRecipient.onNavResult {
        when (it) {
            NavResult.Canceled -> {
                // Do nothing
            }
            is NavResult.Value ->
                if (it.value) {
                    viewModel.deleteList()
                }
        }
    }

    val uiState by viewModel.uiState.collectAsState()
    EditCustomListScreen(
        uiState = uiState,
        onDeleteList = { name ->
            navigator.navigate(DeleteCustomListConfirmationDialogDestination(name)) {
                launchSingleTop = true
            }
        },
        onNameClicked = { id, name -> navigator.navigate(EditCustomListNameDestination(id, name)) },
        onLocationsClicked = {
            navigator.navigate(CustomListLocationsDestination(customListKey = it, newList = false))
        },
        onBackClick = { navigator.navigateUp() }
    )
}

@Composable
fun EditCustomListScreen(
    uiState: EditCustomListState,
    onDeleteList: (name: String) -> Unit = {},
    onNameClicked: (id: String, name: String) -> Unit = { _, _ -> },
    onLocationsClicked: (String) -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    val title =
        when (uiState) {
            is EditCustomListState.Loading -> ""
            is EditCustomListState.Content -> uiState.name
        }
    ScaffoldWithMediumTopBar(
        appBarTitle = title,
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        actions = { Actions(onDeleteList = { onDeleteList(title) }) },
    ) { modifier: Modifier ->
        SpacedColumn(modifier = modifier, alignment = Alignment.Top) {
            when (uiState) {
                EditCustomListState.Loading -> {
                    MullvadCircularProgressIndicatorLarge()
                }
                is EditCustomListState.Content -> {
                    // Name cell
                    TwoRowCell(
                        titleText = stringResource(id = R.string.list_name),
                        subtitleText = uiState.name,
                        onCellClicked = { onNameClicked(uiState.id, uiState.name) }
                    )
                    // Locations cell
                    TwoRowCell(
                        titleText = stringResource(id = R.string.locations),
                        subtitleText =
                            stringResource(
                                id = R.string.number_of_locations,
                                uiState.numberOfLocations
                            ),
                        onCellClicked = { onLocationsClicked(uiState.id) }
                    )
                }
            }
        }
    }
}

@Composable
private fun Actions(onDeleteList: () -> Unit) {
    var showMenu by remember { mutableStateOf(false) }
    IconButton(onClick = { showMenu = true }) {
        Icon(painter = painterResource(id = R.drawable.icon_more_vert), contentDescription = null)
        if (showMenu) {
            DropdownMenu(
                expanded = true,
                onDismissRequest = { showMenu = false },
                modifier = Modifier.background(MaterialTheme.colorScheme.background)
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
                                leadingIconColor = MaterialTheme.colorScheme.onBackground,
                                textColor = MaterialTheme.colorScheme.onBackground,
                            ),
                    onClick = {
                        onDeleteList()
                        showMenu = false
                    }
                )
            }
        }
    }
}

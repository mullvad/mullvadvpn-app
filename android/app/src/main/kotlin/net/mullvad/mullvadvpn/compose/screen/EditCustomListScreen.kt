package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
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
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.CustomListLocationsDestination
import com.ramcosta.composedestinations.generated.destinations.DeleteCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.EditCustomListNameDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.TwoRowCell
import net.mullvad.mullvadvpn.compose.communication.Deleted
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.state.EditCustomListState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.DELETE_DROPDOWN_MENU_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.TOP_BAR_DROPDOWN_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.menuItemColors
import net.mullvad.mullvadvpn.viewmodel.EditCustomListViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewEditCustomListScreen() {
    AppTheme {
        EditCustomListScreen(
            state =
                EditCustomListState.Content(
                    id = CustomListId("id"),
                    name = CustomListName.fromString("Custom list"),
                    locations =
                        listOf(
                            GeoLocationId.Hostname(
                                GeoLocationId.City(
                                    GeoLocationId.Country("country"),
                                    cityCode = "city"
                                ),
                                "hostname",
                            )
                        )
                )
        )
    }
}

data class EditCustomListNavArgs(val customListId: CustomListId)

@Composable
@Destination<RootGraph>(
    style = SlideInFromRightTransition::class,
    navArgs = EditCustomListNavArgs::class
)
fun EditCustomList(
    navigator: DestinationsNavigator,
    backNavigator: ResultBackNavigator<Deleted>,
    confirmDeleteListResultRecipient: ResultRecipient<DeleteCustomListDestination, Deleted>
) {
    val viewModel = koinViewModel<EditCustomListViewModel>()

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
        onDeleteList =
            dropUnlessResumed { id, name ->
                navigator.navigate(
                    DeleteCustomListDestination(customListId = id, name = name),
                )
            },
        onNameClicked =
            dropUnlessResumed { id, name ->
                navigator.navigate(
                    EditCustomListNameDestination(customListId = id, initialName = name),
                )
            },
        onLocationsClicked =
            dropUnlessResumed { id ->
                navigator.navigate(
                    CustomListLocationsDestination(customListId = id, newList = false),
                )
            },
        onBackClick = dropUnlessResumed { backNavigator.navigateBack() }
    )
}

@Composable
fun EditCustomListScreen(
    state: EditCustomListState,
    onDeleteList: (id: CustomListId, name: CustomListName) -> Unit = { _, _ -> },
    onNameClicked: (id: CustomListId, name: CustomListName) -> Unit = { _, _ -> },
    onLocationsClicked: (CustomListId) -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.edit_list),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        actions = {
            val content = state as? EditCustomListState.Content
            Actions(
                enabled = content?.name != null,
                onDeleteList = {
                    if (content is EditCustomListState.Content) {
                        onDeleteList(content.id, content.name)
                    }
                }
            )
        },
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
                        subtitleText = state.name.value,
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
private fun Actions(enabled: Boolean, onDeleteList: () -> Unit) {
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
                    colors = menuItemColors,
                    onClick = {
                        onDeleteList()
                        showMenu = false
                    },
                    enabled = enabled,
                    modifier = Modifier.testTag(DELETE_DROPDOWN_MENU_ITEM_TEST_TAG)
                )
            }
        }
    }
}

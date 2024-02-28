package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.destinations.CreateCustomListDestination
import net.mullvad.mullvadvpn.compose.destinations.CustomListLocationsDestination
import net.mullvad.mullvadvpn.compose.destinations.EditCustomListDestination
import net.mullvad.mullvadvpn.compose.extensions.itemsWithDivider
import net.mullvad.mullvadvpn.compose.state.CustomListsUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.viewmodel.CustomListsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
fun PreviewCustomListsScreen() {
    AppTheme { CustomListsScreen(CustomListsUiState.Content()) }
}

@Composable
@Destination(style = SlideInFromRightTransition::class)
fun CustomLists(
    navigator: DestinationsNavigator,
    createCustomListResultRecipient: ResultRecipient<CreateCustomListDestination, String>
) {
    val viewModel = koinViewModel<CustomListsViewModel>()
    val uiState by viewModel.uiState.collectAsState()

    createCustomListResultRecipient.onNavResult(
        listener = { result ->
            when (result) {
                NavResult.Canceled -> {
                    /* Do nothing */
                }
                is NavResult.Value -> {
                    navigator.navigate(
                        CustomListLocationsDestination(customListKey = result.value, newList = true)
                    )
                }
            }
        }
    )

    CustomListsScreen(
        uiState = uiState,
        addCustomList = { navigator.navigate(CreateCustomListDestination()) },
        openCustomList = { customList ->
            navigator.navigate(EditCustomListDestination(customListId = customList.id))
        },
        onBackClick = navigator::navigateUp
    )
}

@Composable
fun CustomListsScreen(
    uiState: CustomListsUiState,
    addCustomList: () -> Unit = {},
    openCustomList: (RelayItem.CustomList) -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.edit_custom_lists),
        navigationIcon = { NavigateBackIconButton(onBackClick) },
        floatingActionButton = {
            FloatingActionButton(
                onClick = addCustomList,
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
                shape = MaterialTheme.shapes.small
            ) {
                Icon(Icons.Filled.Add, stringResource(id = R.string.create_new_list))
            }
        }
    ) { modifier: Modifier, lazyListState: LazyListState ->
        LazyColumn(
            modifier = modifier,
            state = lazyListState,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            when (uiState) {
                is CustomListsUiState.Content -> {
                    if (uiState.customLists.isNotEmpty()) {
                        content(customLists = uiState.customLists, openCustomList = openCustomList)
                    } else {
                        empty()
                    }
                }
                is CustomListsUiState.Loading -> {
                    loading()
                }
            }
        }
    }
}

private fun LazyListScope.loading() {
    item(contentType = ContentType.PROGRESS) { MullvadCircularProgressIndicatorLarge() }
}

private fun LazyListScope.content(
    customLists: List<RelayItem.CustomList>,
    openCustomList: (RelayItem.CustomList) -> Unit
) {
    itemsWithDivider(
        items = customLists,
        key = { item: RelayItem.CustomList -> item.id },
        contentType = { ContentType.ITEM }
    ) { customList ->
        NavigationComposeCell(title = customList.name, onClick = { openCustomList(customList) })
    }
}

private fun LazyListScope.empty() {
    item(contentType = ContentType.EMPTY_TEXT) {
        Text(
            text = stringResource(R.string.no_custom_lists_available),
            modifier = Modifier.padding(Dimens.screenVerticalMargin),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSecondary
        )
    }
}

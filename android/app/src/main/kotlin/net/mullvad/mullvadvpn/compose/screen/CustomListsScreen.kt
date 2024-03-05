package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.destinations.CreateCustomListDestination
import net.mullvad.mullvadvpn.compose.destinations.EditCustomListDestination
import net.mullvad.mullvadvpn.compose.extensions.itemsWithDivider
import net.mullvad.mullvadvpn.compose.extensions.showSnackbar
import net.mullvad.mullvadvpn.compose.state.CustomListsUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.shape.fabShape
import net.mullvad.mullvadvpn.relaylist.RelayItem
import net.mullvad.mullvadvpn.viewmodel.CustomListsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
fun PreviewCustomListsScreen() {
    AppTheme { CustomListsScreen(CustomListsUiState.Content(), SnackbarHostState()) }
}

@Composable
@Destination(style = SlideInFromRightTransition::class)
fun CustomLists(
    navigator: DestinationsNavigator,
    editCustomListResultRecipient:
        ResultRecipient<EditCustomListDestination, CustomListResult.Deleted>
) {
    val viewModel = koinViewModel<CustomListsViewModel>()
    val uiState by viewModel.uiState.collectAsState()
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    val snackbarHostState = remember { SnackbarHostState() }

    editCustomListResultRecipient.onNavResult { result ->
        when (result) {
            NavResult.Canceled -> {
                /* Do nothing */
            }
            is NavResult.Value -> {
                scope.launch {
                    snackbarHostState.currentSnackbarData?.dismiss()
                    snackbarHostState.showSnackbar(
                        message =
                            context.getString(
                                R.string.delete_custom_list_message,
                                result.value.name
                            ),
                        actionLabel = context.getString(R.string.undo),
                        duration = SnackbarDuration.Long,
                        onAction = { viewModel.undoDeleteCustomList(result.value.undo) }
                    )
                }
            }
        }
    }

    CustomListsScreen(
        uiState = uiState,
        snackbarHostState = snackbarHostState,
        addCustomList = {
            navigator.navigate(
                CreateCustomListDestination(),
            ) {
                launchSingleTop = true
            }
        },
        openCustomList = { customList ->
            navigator.navigate(EditCustomListDestination(customListId = customList.id)) {
                launchSingleTop = true
            }
        },
        onBackClick = navigator::navigateUp
    )
}

@Composable
fun CustomListsScreen(
    uiState: CustomListsUiState,
    snackbarHostState: SnackbarHostState,
    addCustomList: () -> Unit = {},
    openCustomList: (RelayItem.CustomList) -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.edit_custom_lists),
        navigationIcon = { NavigateBackIconButton(onBackClick) },
        floatingActionButton = {
            ExtendedFloatingActionButton(
                onClick = addCustomList,
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
                shape = MaterialTheme.shapes.fabShape
            ) {
                Icon(
                    imageVector = Icons.Filled.Add,
                    contentDescription = stringResource(id = R.string.new_list)
                )
                Spacer(modifier = Modifier.width(Dimens.mediumPadding))
                Text(stringResource(id = R.string.new_list))
            }
        },
        snackbarHostState = snackbarHostState
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

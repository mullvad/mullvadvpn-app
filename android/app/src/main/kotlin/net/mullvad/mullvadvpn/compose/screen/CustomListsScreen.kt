package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
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
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.NEW_LIST_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.Alpha60
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.viewmodel.CustomListsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewCustomListsScreen() {
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
    val state by viewModel.uiState.collectAsStateWithLifecycle()
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
        state = state,
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
    state: CustomListsUiState,
    snackbarHostState: SnackbarHostState,
    addCustomList: () -> Unit = {},
    openCustomList: (CustomList) -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.edit_custom_lists),
        navigationIcon = { NavigateBackIconButton(onBackClick) },
        actions = {
            IconButton(
                onClick = addCustomList,
                modifier = Modifier.testTag(NEW_LIST_BUTTON_TEST_TAG)
            ) {
                Icon(
                    painterResource(id = R.drawable.ic_icons_add),
                    tint =
                        MaterialTheme.colorScheme.onBackground
                            .copy(alpha = Alpha60)
                            .compositeOver(MaterialTheme.colorScheme.background),
                    contentDescription = stringResource(id = R.string.new_list)
                )
            }
        },
        snackbarHostState = snackbarHostState
    ) { modifier: Modifier, lazyListState: LazyListState ->
        LazyColumn(
            modifier = modifier,
            state = lazyListState,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            when (state) {
                is CustomListsUiState.Content -> {
                    if (state.customLists.isNotEmpty()) {
                        content(customLists = state.customLists, openCustomList = openCustomList)
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
    item(contentType = ContentType.PROGRESS) {
        MullvadCircularProgressIndicatorLarge(
            modifier = Modifier.testTag(CIRCULAR_PROGRESS_INDICATOR)
        )
    }
}

private fun LazyListScope.content(
    customLists: List<CustomList>,
    openCustomList: (CustomList) -> Unit
) {
    itemsWithDivider(
        items = customLists,
        key = { item: CustomList -> item.id },
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

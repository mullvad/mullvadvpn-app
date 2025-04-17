package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.CreateCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.EditCustomListDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.compose.communication.Deleted
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.extensions.itemsWithDivider
import net.mullvad.mullvadvpn.compose.preview.CustomListsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.CustomListsUiState
import net.mullvad.mullvadvpn.compose.test.NEW_LIST_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.CustomListsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Content|Empty|Loading")
@Composable
private fun PreviewAccountScreen(
    @PreviewParameter(CustomListsUiStatePreviewParameterProvider::class) state: CustomListsUiState
) {
    AppTheme { CustomListsScreen(state = state, SnackbarHostState(), {}, { _ -> }, {}) }
}

@Composable
@Destination<RootGraph>(style = SlideInFromRightTransition::class)
fun CustomLists(
    navigator: DestinationsNavigator,
    editCustomListResultRecipient:
        ResultRecipient<EditCustomListDestination, CustomListActionResultData.Success.Deleted>,
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
            is NavResult.Value ->
                scope.launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            context.getString(
                                R.string.delete_custom_list_message,
                                result.value.customListName,
                            ),
                        actionLabel = context.getString(R.string.undo),
                        duration = SnackbarDuration.Long,
                        onAction = { viewModel.undoDeleteCustomList(result.value.undo) },
                    )
                }
        }
    }

    CustomListsScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        addCustomList = dropUnlessResumed { navigator.navigate(CreateCustomListDestination(null)) },
        openCustomList =
            dropUnlessResumed { customList ->
                navigator.navigate(EditCustomListDestination(customListId = customList.id))
            },
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
fun CustomListsScreen(
    state: CustomListsUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    addCustomList: () -> Unit,
    openCustomList: (CustomList) -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.edit_custom_lists),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        actions = {
            IconButton(
                onClick = addCustomList,
                modifier = Modifier.testTag(NEW_LIST_BUTTON_TEST_TAG),
            ) {
                Icon(
                    imageVector = Icons.Default.Add,
                    tint = MaterialTheme.colorScheme.onSurface,
                    contentDescription = stringResource(id = R.string.new_list),
                )
            }
        },
        snackbarHostState = snackbarHostState,
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
    item(contentType = ContentType.PROGRESS) { MullvadCircularProgressIndicatorLarge() }
}

private fun LazyListScope.content(
    customLists: List<CustomList>,
    openCustomList: (CustomList) -> Unit,
) {
    itemsWithDivider(
        items = customLists,
        key = { item: CustomList -> item.id },
        contentType = { ContentType.ITEM },
    ) { customList ->
        NavigationComposeCell(
            title = customList.name.value,
            onClick = { openCustomList(customList) },
        )
    }
}

private fun LazyListScope.empty() {
    item(contentType = ContentType.EMPTY_TEXT) {
        Text(
            text = stringResource(R.string.no_custom_lists_available),
            modifier = Modifier.padding(Dimens.screenVerticalMargin),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

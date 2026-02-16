package net.mullvad.mullvadvpn.feature.customlist.impl.screen.lists

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
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
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.generated.customlist.destinations.EditCustomListDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.itemsIndexedWithDivider
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.animation.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.communication.CustomListActionResultData
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.NavigationListItem
import net.mullvad.mullvadvpn.lib.ui.component.positionForIndex
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.NEW_LIST_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel

@Preview("Content|Empty|Loading")
@Composable
private fun PreviewAccountScreen(
    @PreviewParameter(CustomListsUiStatePreviewParameterProvider::class) state: CustomListsUiState
) {
    AppTheme {
        CustomListsScreen(
            state = state,
            snackbarHostState = SnackbarHostState(),
            addCustomList = {},
            openCustomList = { _ -> },
            onBackClick = {},
        )
    }
}

@Composable
@Destination<ExternalModuleGraph>(style = SlideInFromRightTransition::class)
fun CustomLists(
    navigator: DestinationsNavigator,
    editCustomListResultRecipient:
        ResultRecipient<EditCustomListDestination, CustomListActionResultData.Success.Deleted>,
) {
    val viewModel = koinViewModel<CustomListsViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val scope = rememberCoroutineScope()
    val resources = LocalResources.current
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
                            resources.getString(
                                R.string.delete_custom_list_message,
                                result.value.customListName,
                            ),
                        actionLabel = resources.getString(R.string.undo),
                        duration = SnackbarDuration.Long,
                        onAction = { viewModel.undoDeleteCustomList(result.value.undo) },
                    )
                }
        }
    }

    CustomListsScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        addCustomList =
            dropUnlessResumed {
                navigator.navigate(
                    com.ramcosta.composedestinations.generated.customlist.destinations
                        .CreateCustomListDestination(null)
                )
            },
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
                    imageVector = Icons.Rounded.Add,
                    tint = MaterialTheme.colorScheme.onSurface,
                    contentDescription = stringResource(id = R.string.new_list),
                )
            }
        },
        snackbarHostState = snackbarHostState,
    ) { modifier: Modifier, lazyListState: LazyListState ->
        LazyColumn(
            modifier = modifier.padding(horizontal = Dimens.sideMarginNew),
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
    itemsIndexedWithDivider(
        items = customLists,
        key = { _, item: CustomList -> item.id },
        contentType = { _, _ -> ContentType.ITEM },
    ) { index, customList ->
        NavigationListItem(
            title = customList.name.value,
            position = customLists.positionForIndex(index),
            onClick = { openCustomList(customList) },
        )
    }
}

private fun LazyListScope.empty() {
    item(contentType = ContentType.EMPTY_TEXT) {
        Text(
            text = stringResource(R.string.no_custom_lists_available),
            modifier = Modifier.padding(Dimens.mediumPadding),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

object ContentType {
    const val ITEM = 2
    const val PROGRESS = 6
    const val EMPTY_TEXT = 7
}

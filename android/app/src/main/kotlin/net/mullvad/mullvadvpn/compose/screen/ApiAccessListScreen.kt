package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.ApiAccessMethodDetailsDestination
import com.ramcosta.composedestinations.generated.destinations.ApiAccessMethodInfoDestination
import com.ramcosta.composedestinations.generated.destinations.EditApiAccessMethodDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.cell.TwoRowCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.extensions.itemsWithDivider
import net.mullvad.mullvadvpn.compose.preview.ApiAccessListUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.ApiAccessListUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodSetting
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.tag.API_ACCESS_LIST_INFO_TEST_TAG
import net.mullvad.mullvadvpn.viewmodel.ApiAccessListViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Default|WithoutCustomApi|WithCustomApi")
@Composable
private fun PreviewApiAccessList(
    @PreviewParameter(ApiAccessListUiStatePreviewParameterProvider::class)
    state: ApiAccessListUiState
) {
    AppTheme {
        ApiAccessListScreen(
            state = state,
            onAddMethodClick = {},
            onApiAccessMethodClick = { _ -> },
            onApiAccessInfoClick = {},
            onBackClick = {},
        )
    }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun ApiAccessList(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<ApiAccessListViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    ApiAccessListScreen(
        state = state,
        onAddMethodClick = {
            navigator.navigate(EditApiAccessMethodDestination(null)) { launchSingleTop = true }
        },
        onApiAccessMethodClick = {
            navigator.navigate(ApiAccessMethodDetailsDestination(it.id)) { launchSingleTop = true }
        },
        onApiAccessInfoClick = {
            navigator.navigate(ApiAccessMethodInfoDestination) { launchSingleTop = true }
        },
        onBackClick = navigator::navigateUp,
    )
}

@Composable
fun ApiAccessListScreen(
    state: ApiAccessListUiState,
    onAddMethodClick: () -> Unit,
    onApiAccessMethodClick: (apiAccessMethodSetting: ApiAccessMethodSetting) -> Unit,
    onApiAccessInfoClick: () -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings_api_access),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier, lazyListState ->
        LazyColumn(modifier = modifier, state = lazyListState) {
            description()
            currentAccessMethod(
                currentApiAccessMethodName = state.currentApiAccessMethodSetting?.name,
                onInfoClicked = onApiAccessInfoClick,
            )
            apiAccessMethodItems(
                state.apiAccessMethodSettings,
                onApiAccessMethodClick = onApiAccessMethodClick,
            )
            buttonPanel(onAddMethodClick = onAddMethodClick)
        }
    }
}

private fun LazyListScope.description() {
    item {
        Text(
            text = stringResource(id = R.string.api_access_description),
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier =
                Modifier.padding(start = Dimens.mediumPadding, end = Dimens.mediumPadding)
                    .fillMaxWidth(),
        )
    }
}

private fun LazyListScope.currentAccessMethod(
    currentApiAccessMethodName: ApiAccessMethodName?,
    onInfoClicked: () -> Unit,
) {
    item {
        Row(
            modifier =
                Modifier.padding(
                    start = Dimens.mediumPadding,
                    end = Dimens.mediumPadding,
                    bottom = Dimens.mediumPadding,
                ),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
                text =
                    stringResource(
                        id = R.string.current_method,
                        currentApiAccessMethodName?.value ?: "-",
                    ),
            )
            IconButton(
                onClick = onInfoClicked,
                modifier =
                    Modifier.align(Alignment.CenterVertically)
                        .testTag(API_ACCESS_LIST_INFO_TEST_TAG),
            ) {
                Icon(
                    imageVector = Icons.Default.Info,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
        }
    }
}

private fun LazyListScope.apiAccessMethodItems(
    apiAccessMethodSettings: List<ApiAccessMethodSetting>,
    onApiAccessMethodClick: (apiAccessMethodSetting: ApiAccessMethodSetting) -> Unit,
) {
    itemsWithDivider(
        items = apiAccessMethodSettings,
        key = { item -> item.id },
        contentType = { ContentType.ITEM },
    ) {
        ApiAccessMethodItem(
            apiAccessMethodSetting = it,
            onApiAccessMethodClick = onApiAccessMethodClick,
        )
    }
}

@Composable
private fun ApiAccessMethodItem(
    apiAccessMethodSetting: ApiAccessMethodSetting,
    onApiAccessMethodClick: (apiAccessMethodSetting: ApiAccessMethodSetting) -> Unit,
) {
    TwoRowCell(
        titleText = apiAccessMethodSetting.name.value,
        subtitleText =
            stringResource(
                id =
                    if (apiAccessMethodSetting.enabled) {
                        R.string.on
                    } else {
                        R.string.off
                    }
            ),
        bodyView = {
            Icon(
                Icons.AutoMirrored.Default.KeyboardArrowRight,
                contentDescription = apiAccessMethodSetting.name.value,
                tint = MaterialTheme.colorScheme.onPrimary,
            )
        },
        onCellClicked = { onApiAccessMethodClick(apiAccessMethodSetting) },
    )
}

private fun LazyListScope.buttonPanel(onAddMethodClick: () -> Unit) {
    item {
        PrimaryButton(
            modifier =
                Modifier.padding(horizontal = Dimens.sideMargin, vertical = Dimens.largePadding),
            onClick = onAddMethodClick,
            text = stringResource(id = R.string.add),
        )
    }
}

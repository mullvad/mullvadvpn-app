package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.button.TestMethodButton
import net.mullvad.mullvadvpn.compose.cell.HeaderSwitchComposeCell
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.destinations.DeleteApiAccessMethodConfirmationDestination
import net.mullvad.mullvadvpn.compose.destinations.EditApiAccessMethodDestination
import net.mullvad.mullvadvpn.compose.preview.ApiAccessMethodDetailsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.ApiAccessMethodDetailsUiState
import net.mullvad.mullvadvpn.compose.test.API_ACCESS_DETAILS_EDIT_BUTTON
import net.mullvad.mullvadvpn.compose.test.API_ACCESS_DETAILS_TOP_BAR_DROPDOWN_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.API_ACCESS_TEST_METHOD_BUTTON
import net.mullvad.mullvadvpn.compose.test.API_ACCESS_USE_METHOD_BUTTON
import net.mullvad.mullvadvpn.compose.test.DELETE_DROPDOWN_MENU_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.menuItemColors
import net.mullvad.mullvadvpn.viewmodel.ApiAccessMethodDetailsSideEffect
import net.mullvad.mullvadvpn.viewmodel.ApiAccessMethodDetailsViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewApiAccessMethodDetailsScreen(
    @PreviewParameter(ApiAccessMethodDetailsUiStatePreviewParameterProvider::class)
    state: ApiAccessMethodDetailsUiState
) {
    AppTheme { ApiAccessMethodDetailsScreen(state = state) }
}

@Destination(style = SlideInFromRightTransition::class)
@Composable
fun ApiAccessMethodDetails(
    navigator: DestinationsNavigator,
    accessMethodId: ApiAccessMethodId,
    confirmDeleteListResultRecipient:
        ResultRecipient<DeleteApiAccessMethodConfirmationDestination, Boolean>
) {
    val viewModel =
        koinViewModel<ApiAccessMethodDetailsViewModel>(
            parameters = { parametersOf(accessMethodId) }
        )

    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current

    LaunchedEffectCollect(sideEffect = viewModel.uiSideEffect) {
        when (it) {
            ApiAccessMethodDetailsSideEffect.GenericError ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        context.getString(R.string.error_occurred)
                    )
                }
            is ApiAccessMethodDetailsSideEffect.OpenEditPage ->
                navigator.navigate(EditApiAccessMethodDestination(it.apiAccessMethodId)) {
                    launchSingleTop = true
                }
        }
    }

    confirmDeleteListResultRecipient.OnNavResultValue { navigator.navigateUp() }

    val state by viewModel.uiState.collectAsStateWithLifecycle()
    ApiAccessMethodDetailsScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onEditMethodClicked = viewModel::openEditPage,
        onEnableClicked = viewModel::setEnableMethod,
        onTestMethodClicked = viewModel::testMethod,
        onUseMethodClicked = viewModel::setCurrentMethod,
        onDeleteApiAccessMethodClicked = {
            navigator.navigate(DeleteApiAccessMethodConfirmationDestination(it)) {
                launchSingleTop = true
            }
        },
        onBackClicked = navigator::navigateUp
    )
}

@Composable
fun ApiAccessMethodDetailsScreen(
    state: ApiAccessMethodDetailsUiState,
    snackbarHostState: SnackbarHostState = SnackbarHostState(),
    onEditMethodClicked: () -> Unit = {},
    onEnableClicked: (Boolean) -> Unit = {},
    onTestMethodClicked: () -> Unit = {},
    onUseMethodClicked: () -> Unit = {},
    onDeleteApiAccessMethodClicked: (ApiAccessMethodId) -> Unit = {},
    onBackClicked: () -> Unit = {}
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = state.name(),
        navigationIcon = { NavigateBackIconButton(onBackClicked) },
        snackbarHostState = snackbarHostState,
        actions = {
            if (state.canBeEdited()) {
                Actions(
                    onDeleteAccessMethod = {
                        onDeleteApiAccessMethodClicked(state.apiAccessMethodId)
                    }
                )
            }
        }
    ) { modifier: Modifier ->
        Column(modifier = modifier) {
            when (state) {
                is ApiAccessMethodDetailsUiState.Loading -> Loading()
                is ApiAccessMethodDetailsUiState.Content ->
                    Content(
                        state = state,
                        onEditMethodClicked = onEditMethodClicked,
                        onEnableClicked = onEnableClicked,
                        onTestMethodClicked = onTestMethodClicked,
                        onUseMethodClicked = onUseMethodClicked
                    )
            }
        }
    }
}

@Composable
private fun ColumnScope.Loading() {
    MullvadCircularProgressIndicatorLarge(modifier = Modifier.align(Alignment.CenterHorizontally))
}

@Composable
private fun Content(
    state: ApiAccessMethodDetailsUiState.Content,
    onEditMethodClicked: () -> Unit,
    onEnableClicked: (Boolean) -> Unit,
    onTestMethodClicked: () -> Unit,
    onUseMethodClicked: () -> Unit
) {
    if (state.isEditable) {
        NavigationComposeCell(
            title = stringResource(id = R.string.edit_method),
            onClick = onEditMethodClicked,
            testTag = API_ACCESS_DETAILS_EDIT_BUTTON
        )
        HorizontalDivider()
    }
    HeaderSwitchComposeCell(
        isEnabled = state.isDisableable,
        title = stringResource(id = R.string.enable_method),
        isToggled = state.enabled,
        onCellClicked = onEnableClicked
    )
    if (!state.isDisableable) {
        SwitchComposeSubtitleCell(
            text = stringResource(id = R.string.at_least_on_method_needs_to_enabled),
        )
    }
    Spacer(modifier = Modifier.height(Dimens.verticalSpace))
    TestMethodButton(
        modifier =
            Modifier.padding(horizontal = Dimens.sideMargin).testTag(API_ACCESS_TEST_METHOD_BUTTON),
        testMethodState = state.testApiAccessMethodState,
        onTestMethod = onTestMethodClicked
    )
    Spacer(modifier = Modifier.height(Dimens.verticalSpace))
    PrimaryButton(
        isEnabled = !state.isCurrentMethod,
        modifier =
            Modifier.padding(horizontal = Dimens.sideMargin).testTag(API_ACCESS_USE_METHOD_BUTTON),
        onClick = onUseMethodClicked,
        text = stringResource(id = R.string.use_method)
    )
    if (state.isCurrentMethod) {
        AlreadyCurrentText()
    }
}

@Composable
private fun Actions(onDeleteAccessMethod: () -> Unit) {
    var showMenu by remember { mutableStateOf(false) }
    IconButton(
        onClick = { showMenu = true },
        modifier = Modifier.testTag(API_ACCESS_DETAILS_TOP_BAR_DROPDOWN_BUTTON_TEST_TAG)
    ) {
        Icon(painter = painterResource(id = R.drawable.icon_more_vert), contentDescription = null)
        if (showMenu) {
            DropdownMenu(
                expanded = true,
                onDismissRequest = { showMenu = false },
                modifier = Modifier.background(MaterialTheme.colorScheme.surfaceContainer)
            ) {
                DropdownMenuItem(
                    text = { Text(text = stringResource(id = R.string.delete_method)) },
                    leadingIcon = {
                        Icon(
                            painter = painterResource(id = R.drawable.icon_delete),
                            contentDescription = null,
                        )
                    },
                    colors = menuItemColors,
                    onClick = {
                        onDeleteAccessMethod()
                        showMenu = false
                    },
                    modifier = Modifier.testTag(DELETE_DROPDOWN_MENU_ITEM_TEST_TAG)
                )
            }
        }
    }
}

@Composable
private fun AlreadyCurrentText() {
    Text(
        modifier = Modifier.padding(horizontal = Dimens.largePadding),
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSecondary,
        text = stringResource(id = R.string.this_is_already_set_as_current),
    )
}

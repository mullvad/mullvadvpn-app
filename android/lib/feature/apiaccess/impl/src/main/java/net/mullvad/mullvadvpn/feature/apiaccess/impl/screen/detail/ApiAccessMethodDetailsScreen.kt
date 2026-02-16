package net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.detail

import android.content.res.Resources
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.MoreVert
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
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
import com.ramcosta.composedestinations.generated.apiaccess.destinations.DeleteApiAccessMethodConfirmationDestination
import com.ramcosta.composedestinations.generated.apiaccess.destinations.EncryptedDnsProxyInfoDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.OnNavResultValue
import net.mullvad.mullvadvpn.core.animation.SlideInFromRightTransition
import net.mullvad.mullvadvpn.feature.apiaccess.impl.component.TestMethodButton
import net.mullvad.mullvadvpn.feature.apiaccess.impl.util.toDisplayName
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.NavigationListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.ListItemInfo
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.API_ACCESS_DETAILS_EDIT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.API_ACCESS_DETAILS_TOP_BAR_DROPDOWN_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.API_ACCESS_TEST_METHOD_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.API_ACCESS_USE_METHOD_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.DELETE_DROPDOWN_MENU_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.menuItemColors
import org.koin.androidx.compose.koinViewModel

@Preview("Loading|NonEditable|Editable")
@Composable
private fun PreviewApiAccessMethodDetailsScreen(
    @PreviewParameter(ApiAccessMethodDetailsUiStatePreviewParameterProvider::class)
    state: ApiAccessMethodDetailsUiState
) {
    AppTheme {
        ApiAccessMethodDetailsScreen(
            state = state,
            snackbarHostState = SnackbarHostState(),
            onEditMethodClicked = {},
            onEnableClicked = {},
            onTestMethodClicked = {},
            onUseMethodClicked = {},
            onDeleteApiAccessMethodClicked = {},
            onNavigateToEncryptedDnsInfoDialog = {},
            onBackClicked = {},
        )
    }
}

data class ApiAccessMethodDetailsNavArgs(val accessMethodId: ApiAccessMethodId)

@Destination<ExternalModuleGraph>(
    style = SlideInFromRightTransition::class,
    navArgs = ApiAccessMethodDetailsNavArgs::class,
)
@Composable
@Suppress("LongMethod")
fun ApiAccessMethodDetails(
    navigator: DestinationsNavigator,
    confirmDeleteListResultRecipient:
        ResultRecipient<DeleteApiAccessMethodConfirmationDestination, Boolean>,
) {
    val viewModel = koinViewModel<ApiAccessMethodDetailsViewModel>()

    val snackbarHostState = remember { SnackbarHostState() }
    val resources = LocalResources.current
    val coroutineScope = rememberCoroutineScope()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            ApiAccessMethodDetailsSideEffect.GenericError ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        resources.getString(R.string.error_occurred)
                    )
                }
            is ApiAccessMethodDetailsSideEffect.OpenEditPage ->
                navigator.navigate(
                    com.ramcosta.composedestinations.generated.apiaccess.destinations
                        .EditApiAccessMethodDestination(it.apiAccessMethodId)
                ) {
                    launchSingleTop = true
                }
            is ApiAccessMethodDetailsSideEffect.TestApiAccessMethodResult -> {
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        resources.getString(
                            if (it.successful) {
                                R.string.api_reachable
                            } else {
                                R.string.api_unreachable
                            }
                        )
                    )
                }
            }
            is ApiAccessMethodDetailsSideEffect.UnableToSetCurrentMethod ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        resources.getString(
                            if (it.testMethodFailed) {
                                R.string.failed_to_set_current_test_error
                            } else {
                                R.string.failed_to_set_current_unknown_error
                            }
                        )
                    )
                }
        }
    }

    confirmDeleteListResultRecipient.OnNavResultValue { navigator.navigateUp() }

    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(state.testingAccessMethod()) {
        if (state.testingAccessMethod()) {
            launch {
                snackbarHostState.showSnackbarImmediately(
                    message = resources.getString(R.string.testing_name, state.name(resources)),
                    duration = SnackbarDuration.Indefinite,
                    actionLabel = resources.getString(R.string.cancel),
                    onAction = viewModel::cancelTestMethod,
                )
            }
        }
    }

    ApiAccessMethodDetailsScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onEditMethodClicked = viewModel::openEditPage,
        onEnableClicked = viewModel::setEnableMethod,
        onTestMethodClicked = viewModel::testMethod,
        onUseMethodClicked = {
            if (!state.currentMethod()) {
                viewModel.setCurrentMethod()
            } else {
                coroutineScope.launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = resources.getString(R.string.this_is_already_set_as_current)
                    )
                }
            }
        },
        onDeleteApiAccessMethodClicked = {
            navigator.navigate(
                com.ramcosta.composedestinations.generated.apiaccess.destinations
                    .DeleteApiAccessMethodConfirmationDestination(it)
            ) {
                launchSingleTop = true
            }
        },
        onNavigateToEncryptedDnsInfoDialog =
            dropUnlessResumed { navigator.navigate(EncryptedDnsProxyInfoDestination) },
        onBackClicked = navigator::navigateUp,
    )
}

@Composable
fun ApiAccessMethodDetailsScreen(
    state: ApiAccessMethodDetailsUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onEditMethodClicked: () -> Unit,
    onEnableClicked: (Boolean) -> Unit,
    onTestMethodClicked: () -> Unit,
    onUseMethodClicked: () -> Unit,
    onDeleteApiAccessMethodClicked: (ApiAccessMethodId) -> Unit,
    onNavigateToEncryptedDnsInfoDialog: () -> Unit,
    onBackClicked: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = state.name(),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClicked) },
        snackbarHostState = snackbarHostState,
        actions = {
            if (state.canBeEdited()) {
                Actions(
                    onDeleteAccessMethod = {
                        onDeleteApiAccessMethodClicked(state.apiAccessMethodId)
                    }
                )
            }
        },
    ) { modifier: Modifier ->
        Column(modifier = modifier.padding(horizontal = Dimens.sideMarginNew)) {
            when (state) {
                is ApiAccessMethodDetailsUiState.Loading -> Loading()
                is ApiAccessMethodDetailsUiState.Content ->
                    Content(
                        state = state,
                        onEditMethodClicked = onEditMethodClicked,
                        onEnableClicked = onEnableClicked,
                        onTestMethodClicked = onTestMethodClicked,
                        onUseMethodClicked = onUseMethodClicked,
                        onNavigateToEncryptedDnsInfoDialog = onNavigateToEncryptedDnsInfoDialog,
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
    onUseMethodClicked: () -> Unit,
    onNavigateToEncryptedDnsInfoDialog: () -> Unit,
) {
    if (state.isEditable) {
        NavigationListItem(
            title = stringResource(id = R.string.edit_method),
            onClick = onEditMethodClicked,
            testTag = API_ACCESS_DETAILS_EDIT_BUTTON_TEST_TAG,
            position = Position.Top,
        )
        HorizontalDivider()
    }
    SwitchListItem(
        isEnabled = state.isDisableable,
        title = stringResource(id = R.string.enable_method),
        isToggled = state.enabled,
        onCellClicked = onEnableClicked,
        onInfoClicked =
            when (state.apiAccessMethod) {
                ApiAccessMethod.EncryptedDns -> onNavigateToEncryptedDnsInfoDialog
                ApiAccessMethod.Bridges,
                is ApiAccessMethod.CustomProxy,
                ApiAccessMethod.Direct -> null
            },
        position =
            if (state.isEditable) {
                Position.Bottom
            } else {
                Position.Single
            },
    )
    if (!state.isDisableable) {
        ListItemInfo(text = stringResource(id = R.string.at_least_on_method_needs_to_enabled))
    }
    Spacer(modifier = Modifier.height(Dimens.largePadding))
    TestMethodButton(
        modifier =
            Modifier.padding(horizontal = Dimens.mediumPadding)
                .testTag(API_ACCESS_TEST_METHOD_BUTTON_TEST_TAG),
        isTesting = state.isTestingAccessMethod,
        onTestMethod = onTestMethodClicked,
    )
    Spacer(modifier = Modifier.height(Dimens.largeSpacer))
    PrimaryButton(
        isEnabled = !state.isTestingAccessMethod,
        modifier =
            Modifier.padding(horizontal = Dimens.mediumPadding)
                .testTag(API_ACCESS_USE_METHOD_BUTTON_TEST_TAG),
        onClick = onUseMethodClicked,
        text = stringResource(id = R.string.use_method),
    )
}

@Composable
private fun Actions(onDeleteAccessMethod: () -> Unit) {
    var showMenu by remember { mutableStateOf(false) }
    IconButton(
        onClick = { showMenu = true },
        modifier = Modifier.testTag(API_ACCESS_DETAILS_TOP_BAR_DROPDOWN_BUTTON_TEST_TAG),
    ) {
        Icon(imageVector = Icons.Rounded.MoreVert, contentDescription = null)
        if (showMenu) {
            DropdownMenu(
                expanded = true,
                onDismissRequest = { showMenu = false },
                modifier = Modifier.background(MaterialTheme.colorScheme.surfaceContainer),
            ) {
                DropdownMenuItem(
                    text = { Text(text = stringResource(id = R.string.delete_method)) },
                    leadingIcon = {
                        Icon(
                            imageVector = Icons.Rounded.Delete,
                            tint = MaterialTheme.colorScheme.onSurface,
                            contentDescription = null,
                        )
                    },
                    colors = menuItemColors,
                    onClick = {
                        onDeleteAccessMethod()
                        showMenu = false
                    },
                    modifier = Modifier.testTag(DELETE_DROPDOWN_MENU_ITEM_TEST_TAG),
                )
            }
        }
    }
}

@Composable
private fun ApiAccessMethodDetailsUiState.name(): String =
    (this as? ApiAccessMethodDetailsUiState.Content)?.apiAccessMethodSetting?.toDisplayName() ?: ""

private fun ApiAccessMethodDetailsUiState.name(resources: Resources): String =
    (this as? ApiAccessMethodDetailsUiState.Content)
        ?.apiAccessMethodSetting
        ?.toDisplayName(resources = resources) ?: ""

package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.cell.DefaultNavigationView
import net.mullvad.mullvadvpn.compose.cell.TwoRowCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.destinations.ApiAccessMethodInfoDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.EditApiAccessMethodDestination
import net.mullvad.mullvadvpn.compose.preview.ApiAccessListUiStateParameterProvider
import net.mullvad.mullvadvpn.compose.state.ApiAccessListUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.ApiAccessListViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewApiAccessList(
    @PreviewParameter(ApiAccessListUiStateParameterProvider::class) state: ApiAccessListUiState
) {
    AppTheme { ApiAccessListScreen(state = state) }
}

@Destination(style = SlideInFromRightTransition::class)
@Composable
fun ApiAccessList(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<ApiAccessListViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    ApiAccessListScreen(
        state = state,
        onAddMethodClick = {
            navigator.navigate(EditApiAccessMethodDestination(null)) { launchSingleTop = true }
        },
        onApiAccessMethodClick = {},
        onApiAccessInfoClick = {
            navigator.navigate(ApiAccessMethodInfoDialogDestination) { launchSingleTop = true }
        },
        onBackClick = { navigator.navigateUp() }
    )
}

@Composable
fun ApiAccessListScreen(
    state: ApiAccessListUiState,
    onAddMethodClick: () -> Unit = {},
    onApiAccessMethodClick: (apiAccessMethod: ApiAccessMethod) -> Unit = {},
    onApiAccessInfoClick: () -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings_api_access),
        navigationIcon = { NavigateBackIconButton(onBackClick) },
    ) { modifier ->
        Column(modifier = modifier) {
            Description()
            CurrentAccessMethod(
                currentApiAccessMethodName = state.currentApiAccessMethod?.name,
                onInfoClicked = onApiAccessInfoClick
            )
            Spacer(modifier = Modifier.height(Dimens.verticalSpace))
            state.apiAccessMethods.forEachIndexed { index, apiAccessMethod ->
                if (index != 0) {
                    HorizontalDivider()
                }
                ApiAccessMethodItem(
                    apiAccessMethod = apiAccessMethod,
                    onApiAccessMethodClick = onApiAccessMethodClick
                )
            }
            Spacer(modifier = Modifier.height(Dimens.verticalSpace))
            ButtonPanel(onAddMethodClick = onAddMethodClick)
        }
    }
}

@Composable
fun Description() {
    Text(
        text = stringResource(id = R.string.api_access_description),
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSecondary,
        modifier =
            Modifier.padding(
                    start = Dimens.cellStartPadding,
                    top = Dimens.cellFooterTopPadding,
                    end = Dimens.cellEndPadding,
                    bottom = Dimens.cellLabelVerticalPadding
                )
                .fillMaxWidth()
                .wrapContentHeight()
    )
}

@Composable
fun CurrentAccessMethod(
    currentApiAccessMethodName: ApiAccessMethodName?,
    onInfoClicked: () -> Unit
) {
    Row(
        modifier =
            Modifier.padding(
                start = Dimens.sideMargin,
                end = Dimens.sideMargin,
            ),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.onBackground,
            text =
                stringResource(
                    id = R.string.current_x,
                    currentApiAccessMethodName?.value ?: "-",
                ),
        )
        IconButton(
            onClick = onInfoClicked,
            modifier = Modifier.align(Alignment.CenterVertically),
        ) {
            Icon(
                painter = painterResource(id = R.drawable.icon_info),
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onBackground
            )
        }
    }
}

@Composable
fun ApiAccessMethodItem(
    apiAccessMethod: ApiAccessMethod,
    onApiAccessMethodClick: (apiAccessMethod: ApiAccessMethod) -> Unit
) {
    TwoRowCell(
        titleText = apiAccessMethod.name.value,
        subtitleText =
            stringResource(
                id =
                    if (apiAccessMethod.enabled) {
                        R.string.on
                    } else {
                        R.string.off
                    }
            ),
        bodyView = { DefaultNavigationView(apiAccessMethod.name.value) },
        onCellClicked = { onApiAccessMethodClick(apiAccessMethod) }
    )
}

@Composable
fun ButtonPanel(onAddMethodClick: () -> Unit) {
    PrimaryButton(
        modifier = Modifier.padding(horizontal = Dimens.sideMargin),
        onClick = onAddMethodClick,
        text = stringResource(id = R.string.add)
    )
}

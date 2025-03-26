package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material.icons.filled.Error
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.ChangelogDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.cell.TwoRowCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.createUriHook
import net.mullvad.mullvadvpn.compose.preview.AppInfoUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.AppInfoSideEffect
import net.mullvad.mullvadvpn.viewmodel.AppInfoUiState
import net.mullvad.mullvadvpn.viewmodel.AppInfoViewModel
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview("Initial|Unsupported")
@Composable
private fun PreviewAppInfoScreen(
    @PreviewParameter(AppInfoUiStatePreviewParameterProvider::class) state: AppInfoUiState
) {
    AppTheme {
        AppInfo(
            state = state,
            snackbarHostState = SnackbarHostState(),
            onBackClick = {},
            navigateToChangelog = {},
            openAppListing = {},
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun AppInfo(navigator: DestinationsNavigator) {
    val vm = koinViewModel<AppInfoViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

    val uriHandler = LocalUriHandler.current
    val snackbarHostState = remember { SnackbarHostState() }

    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            is AppInfoSideEffect.OpenUri -> {
                uriHandler
                    .createUriHook(
                        it.uri.toString(),
                        {
                            launch {
                                snackbarHostState.showSnackbarImmediately(message = it.errorMessage)
                            }
                        },
                    )
                    .invoke()
            }
        }
    }

    AppInfo(
        state = state,
        snackbarHostState = snackbarHostState,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        navigateToChangelog =
            dropUnlessResumed { navigator.navigate(ChangelogDestination(ChangelogNavArgs())) },
        openAppListing = dropUnlessResumed { vm.openAppListing() },
    )
}

@ExperimentalMaterial3Api
@Composable
fun AppInfo(
    state: AppInfoUiState,
    snackbarHostState: SnackbarHostState,
    onBackClick: () -> Unit,
    navigateToChangelog: () -> Unit,
    openAppListing: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.app_info),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        snackbarHostState = snackbarHostState,
    ) { modifier ->
        Column(horizontalAlignment = Alignment.Start, modifier = modifier.animateContentSize()) {
            AppInfoContent(state, navigateToChangelog, openAppListing)
        }
    }
}

@Composable
fun AppInfoContent(
    state: AppInfoUiState,
    navigateToChangelog: () -> Unit,
    openAppListing: () -> Unit,
) {
    Column(modifier = Modifier.padding(bottom = Dimens.smallPadding).animateContentSize()) {
        ChangelogRow(navigateToChangelog)
        HorizontalDivider()
        AppVersionRow(state, openAppListing)
    }
}

@Composable
private fun AppVersionRow(state: AppInfoUiState, openAppListing: () -> Unit) {
    Column {
        TwoRowCell(
            titleText = stringResource(id = R.string.version),
            subtitleText = state.version.currentVersion,
            iconView = {
                if (!state.version.isSupported) {
                    Icon(
                        imageVector = Icons.Default.Error,
                        modifier = Modifier.padding(end = Dimens.smallPadding),
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.error,
                    )
                }
            },
            bodyView = {
                Icon(
                    Icons.AutoMirrored.Default.OpenInNew,
                    contentDescription = stringResource(R.string.app_info),
                    tint = MaterialTheme.colorScheme.onPrimary,
                )
            },
            onCellClicked = openAppListing,
        )

        if (!state.version.isSupported) {
            Text(
                text = stringResource(id = R.string.unsupported_version_description),
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier =
                    Modifier.fillMaxWidth()
                        .padding(
                            start = Dimens.cellStartPadding,
                            end = Dimens.cellStartPadding,
                            top = Dimens.smallPadding,
                            bottom = Dimens.mediumPadding,
                        ),
            )
        }
    }
}

@Composable
private fun ChangelogRow(navigateToChangelog: () -> Unit) {
    NavigationComposeCell(
        title = stringResource(R.string.changelog_title),
        onClick = navigateToChangelog,
    )
}

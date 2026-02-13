package net.mullvad.mullvadvpn.feature.home.impl.devicerevoked

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
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
import androidx.lifecycle.compose.dropUnlessResumed
import androidx.navigation.NavController
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.generated.login.destinations.LoginDestination
import com.ramcosta.composedestinations.generated.settings.destinations.SettingsDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import org.koin.androidx.compose.koinViewModel

@Preview("Secured|Unsecured|Unknown")
@Composable
private fun PreviewDeviceRevokedScreen(
    @PreviewParameter(DeviceRevokedUiStatePreviewParameterProvider::class)
    state: DeviceRevokedUiState
) {
    AppTheme { DeviceRevokedScreen(state = state, onSettingsClicked = {}, onGoToLoginClicked = {}) }
}

@Destination<ExternalModuleGraph>
@Composable
fun DeviceRevoked(navController: NavController, navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<DeviceRevokedViewModel>()

    val state by viewModel.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            DeviceRevokedSideEffect.NavigateToLogin ->
                navController.navigate(LoginDestination.baseRoute) {
                    launchSingleTop = true
                    popUpTo("main") { inclusive = true }
                }
        }
    }

    DeviceRevokedScreen(
        state = state,
        onSettingsClicked = dropUnlessResumed { navigator.navigate(SettingsDestination) },
        onGoToLoginClicked = viewModel::onGoToLoginClicked,
    )
}

@Composable
fun DeviceRevokedScreen(
    state: DeviceRevokedUiState,
    onSettingsClicked: () -> Unit,
    onGoToLoginClicked: () -> Unit,
) {
    val topColor =
        if (state == DeviceRevokedUiState.SECURED) {
            MaterialTheme.colorScheme.tertiary
        } else {
            MaterialTheme.colorScheme.error
        }

    val scrollState = rememberScrollState()
    ScaffoldWithTopBar(
        topBarColor = topColor,
        onSettingsClicked = onSettingsClicked,
        onAccountClicked = null,
    ) {
        Column(
            modifier =
                Modifier.fillMaxSize()
                    .padding(it)
                    .padding(
                        top = Dimens.screenTopMargin,
                        bottom = Dimens.screenBottomMargin,
                        start = Dimens.screenTopMargin,
                        end = Dimens.sideMargin,
                    )
                    .verticalScroll(scrollState)
                    .drawVerticalScrollbar(
                        state = scrollState,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                    )
                    .background(color = MaterialTheme.colorScheme.surface)
        ) {
            Image(
                painter = painterResource(id = R.drawable.icon_fail),
                contentDescription = null,
                modifier =
                    Modifier.align(Alignment.CenterHorizontally)
                        .padding(bottom = Dimens.mediumSpacer),
            )
            Text(
                text = stringResource(id = R.string.device_inactive_title),
                style = MaterialTheme.typography.headlineSmall,
                color = MaterialTheme.colorScheme.onSurface,
            )

            Text(
                text = stringResource(id = R.string.device_inactive_description),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.padding(top = Dimens.smallPadding),
            )

            if (state == DeviceRevokedUiState.SECURED) {
                Text(
                    text = stringResource(id = R.string.device_inactive_unblock_warning),
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.padding(top = Dimens.mediumPadding),
                )
            }

            Spacer(modifier = Modifier.weight(1f).defaultMinSize(minHeight = Dimens.verticalSpace))

            // Button area
            DeviceRevokedLoginButton(onClick = onGoToLoginClicked, state = state)
        }
    }
}

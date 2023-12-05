package net.mullvad.mullvadvpn.compose.screen

import android.window.SplashScreen
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootNavGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.popUpTo
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.NavGraphs
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.destinations.ConnectDestination
import net.mullvad.mullvadvpn.compose.destinations.DeviceRevokedDestination
import net.mullvad.mullvadvpn.compose.destinations.LoginDestination
import net.mullvad.mullvadvpn.compose.destinations.OutOfTimeDestination
import net.mullvad.mullvadvpn.compose.destinations.PrivacyDisclaimerDestination
import net.mullvad.mullvadvpn.compose.transitions.DefaultTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription
import net.mullvad.mullvadvpn.viewmodel.SplashUiSideEffect
import net.mullvad.mullvadvpn.viewmodel.SplashViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewLoadingScreen() {
    AppTheme { SplashScreen() }
}

// Set this as the start destination of the default nav graph
@RootNavGraph(start = true)
@Destination(style = DefaultTransition::class)
@Composable
fun Splash(navigator: DestinationsNavigator) {
    val viewModel: SplashViewModel = koinViewModel()

    LaunchedEffect(Unit) {
        viewModel.uiSideEffect.collect {
            when (it) {
                SplashUiSideEffect.NavigateToConnect -> {
                    navigator.navigate(ConnectDestination) {
                        popUpTo(NavGraphs.root) { inclusive = true }
                    }
                }
                SplashUiSideEffect.NavigateToLogin -> {
                    navigator.navigate(LoginDestination()) {
                        popUpTo(NavGraphs.root) { inclusive = true }
                    }
                }
                SplashUiSideEffect.NavigateToPrivacyDisclaimer -> {
                    navigator.navigate(PrivacyDisclaimerDestination) { popUpTo(NavGraphs.root) {} }
                }
                SplashUiSideEffect.NavigateToRevoked -> {
                    navigator.navigate(DeviceRevokedDestination) {
                        popUpTo(NavGraphs.root) { inclusive = true }
                    }
                }
                SplashUiSideEffect.NavigateToOutOfTime ->
                    navigator.navigate(OutOfTimeDestination) {
                        popUpTo(NavGraphs.root) { inclusive = true }
                    }
            }
        }
    }

    LaunchedEffect(Unit) { viewModel.start() }

    SplashScreen()
}

@Composable
fun SplashScreen() {

    val backgroundColor = MaterialTheme.colorScheme.primary

    ScaffoldWithTopBar(
        topBarColor = backgroundColor,
        onSettingsClicked = null,
        onAccountClicked = null,
        isIconAndLogoVisible = false,
        content = {
            Box(
                contentAlignment = Alignment.Center,
                modifier =
                    Modifier.background(backgroundColor)
                        .padding(it)
                        .padding(bottom = it.calculateTopPadding())
                        .fillMaxSize()
            ) {
                Column(
                    verticalArrangement = Arrangement.Center,
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Image(
                        painter = painterResource(id = R.drawable.launch_logo),
                        contentDescription = "",
                        modifier = Modifier.size(Dimens.splashLogoSize)
                    )
                    Image(
                        painter = painterResource(id = R.drawable.logo_text),
                        contentDescription = "",
                        alpha = 0.6f,
                        modifier =
                            Modifier.padding(top = Dimens.mediumPadding)
                                .height(Dimens.splashLogoTextHeight)
                    )
                    Text(
                        text = stringResource(id = R.string.connecting_to_daemon),
                        style = MaterialTheme.typography.bodySmall,
                        color =
                            MaterialTheme.colorScheme.onPrimary
                                .copy(alpha = AlphaDescription)
                                .compositeOver(backgroundColor),
                        modifier = Modifier.padding(top = Dimens.mediumPadding)
                    )
                }
            }
        }
    )
}

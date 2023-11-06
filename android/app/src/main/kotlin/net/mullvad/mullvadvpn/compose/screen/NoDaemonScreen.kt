package net.mullvad.mullvadvpn.compose.screen

import androidx.activity.compose.BackHandler
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.app.ActivityCompat.finishAffinity
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import kotlinx.coroutines.flow.first
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.destinations.SettingsDestination
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription
import net.mullvad.mullvadvpn.util.getActivity
import net.mullvad.mullvadvpn.viewmodel.ServiceConnectionViewModel
import net.mullvad.mullvadvpn.viewmodel.ServiceState
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewNoDaemonScreen() {
    AppTheme { NoDaemonScreen({}) }
}

// Set this as the start destination of the default nav graph
@Destination
@Composable
fun NoDaemonScreen(navigator: DestinationsNavigator) {
    NoDaemonScreen { navigator.navigate(SettingsDestination) }
}

@Composable
fun NoDaemonScreen(onNavigateToSettings: () -> Unit) {

    val backgroundColor = MaterialTheme.colorScheme.primary

    val context = LocalContext.current
    BackHandler { finishAffinity(context.getActivity()!!) }

    ScaffoldWithTopBar(
        topBarColor = backgroundColor,
        onSettingsClicked = onNavigateToSettings,
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
                        modifier = Modifier.size(120.dp)
                    )
                    Image(
                        painter = painterResource(id = R.drawable.logo_text),
                        contentDescription = "",
                        alpha = 0.6f,
                        modifier = Modifier.padding(top = 12.dp).height(18.dp)
                    )
                    Text(
                        text = stringResource(id = R.string.connecting_to_daemon),
                        fontSize = 13.sp,
                        color =
                            MaterialTheme.colorScheme.onPrimary
                                .copy(alpha = AlphaDescription)
                                .compositeOver(backgroundColor),
                        modifier = Modifier.padding(top = 12.dp)
                    )
                }
            }
        }
    )
}

package net.mullvad.mullvadvpn.compose.screen

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Divider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.popUpTo
import net.mullvad.mullvadvpn.compose.map.Map
import net.mullvad.mullvadvpn.compose.map.data.LatLng
import net.mullvad.mullvadvpn.compose.map.data.Marker
import net.mullvad.mullvadvpn.compose.map.data.MarkerType
import net.mullvad.mullvadvpn.compose.map.data.gothenburgLatLng
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.NavGraphs
import net.mullvad.mullvadvpn.compose.button.ConnectionButton
import net.mullvad.mullvadvpn.compose.button.SwitchLocationButton
import net.mullvad.mullvadvpn.compose.component.ConnectionStatusText
import net.mullvad.mullvadvpn.compose.component.LocationInfo
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBarAndDeviceName
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.component.notificationbanner.NotificationBanner
import net.mullvad.mullvadvpn.compose.destinations.AccountDestination
import net.mullvad.mullvadvpn.compose.destinations.DeviceRevokedDestination
import net.mullvad.mullvadvpn.compose.destinations.OutOfTimeDestination
import net.mullvad.mullvadvpn.compose.destinations.SelectLocationDestination
import net.mullvad.mullvadvpn.compose.destinations.SettingsDestination
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.CONNECT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LOCATION_INFO_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.RECONNECT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SCROLLABLE_COLUMN_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.HomeTransition
import net.mullvad.mullvadvpn.lib.common.util.openAccountPageInBrowser
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.theme.color.AlphaTopBar
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.util.appendHideNavOnPlayBuild
import net.mullvad.mullvadvpn.viewmodel.ConnectViewModel
import org.koin.androidx.compose.koinViewModel

private const val CONNECT_BUTTON_THROTTLE_MILLIS = 1000

@Preview
@Composable
private fun PreviewConnectScreen() {
    val state = ConnectUiState.INITIAL
    AppTheme {
        ConnectScreen(
            uiState = state,
        )
    }
}

@Destination(style = HomeTransition::class)
@Composable
fun Connect(navigator: DestinationsNavigator) {
    val connectViewModel: ConnectViewModel = koinViewModel()

    val state = connectViewModel.uiState.collectAsState().value

    val context = LocalContext.current
    LaunchedEffect(key1 = Unit) {
        connectViewModel.uiSideEffect.collect { uiSideEffect ->
            when (uiSideEffect) {
                is ConnectViewModel.UiSideEffect.OpenAccountManagementPageInBrowser -> {
                    context.openAccountPageInBrowser(uiSideEffect.token)
                }
                is ConnectViewModel.UiSideEffect.OutOfTime -> {
                    navigator.navigate(OutOfTimeDestination) {
                        launchSingleTop = true
                        popUpTo(NavGraphs.root) { inclusive = true }
                    }
                }
                ConnectViewModel.UiSideEffect.RevokedDevice -> {
                    navigator.navigate(DeviceRevokedDestination) {
                        launchSingleTop = true
                        popUpTo(NavGraphs.root) { inclusive = true }
                    }
                }
            }
        }
    }
    ConnectScreen(
        uiState = state,
        onDisconnectClick = connectViewModel::onDisconnectClick,
        onReconnectClick = connectViewModel::onReconnectClick,
        onConnectClick = connectViewModel::onConnectClick,
        onCancelClick = connectViewModel::onCancelClick,
        onSwitchLocationClick = {
            navigator.navigate(SelectLocationDestination) { launchSingleTop = true }
        },
        onUpdateVersionClick = {
            val intent =
                Intent(
                        Intent.ACTION_VIEW,
                        Uri.parse(
                            context
                                .getString(R.string.download_url)
                                .appendHideNavOnPlayBuild(state.isPlayBuild)
                        )
                    )
                    .apply { flags = Intent.FLAG_ACTIVITY_NEW_TASK }
            context.startActivity(intent)
        },
        onManageAccountClick = connectViewModel::onManageAccountClick,
        onSettingsClick = { navigator.navigate(SettingsDestination) { launchSingleTop = true } },
        onAccountClick = { navigator.navigate(AccountDestination) { launchSingleTop = true } },
        onDismissNewDeviceClick = connectViewModel::dismissNewDeviceNotification,
    )
}

@Composable
fun ConnectScreen(
    uiState: ConnectUiState,
    onDisconnectClick: () -> Unit = {},
    onReconnectClick: () -> Unit = {},
    onConnectClick: () -> Unit = {},
    onCancelClick: () -> Unit = {},
    onSwitchLocationClick: () -> Unit = {},
    onUpdateVersionClick: () -> Unit = {},
    onManageAccountClick: () -> Unit = {},
    onSettingsClick: () -> Unit = {},
    onAccountClick: () -> Unit = {},
    onDismissNewDeviceClick: () -> Unit = {}
) {

    val scrollState = rememberScrollState()
    var lastConnectionActionTimestamp by remember { mutableLongStateOf(0L) }

    fun handleThrottledAction(action: () -> Unit) {
        val currentTime = System.currentTimeMillis()
        if ((currentTime - lastConnectionActionTimestamp) > CONNECT_BUTTON_THROTTLE_MILLIS) {
            lastConnectionActionTimestamp = currentTime
            action.invoke()
        }
    }

    ScaffoldWithTopBarAndDeviceName(
        topBarColor =
            if (uiState.tunnelUiState.isSecured()) {
                MaterialTheme.colorScheme.inversePrimary
            } else {
                MaterialTheme.colorScheme.error
            },
        iconTintColor =
            if (uiState.tunnelUiState.isSecured()) {
                    MaterialTheme.colorScheme.onPrimary
                } else {
                    MaterialTheme.colorScheme.onError
                }
                .copy(alpha = AlphaTopBar),
        onSettingsClicked = onSettingsClick,
        onAccountClicked = onAccountClick,
        deviceName = uiState.deviceName,
        timeLeft = uiState.daysLeftUntilExpiry
    ) {
        var bias by remember { mutableFloatStateOf(0.5f) }
        Map(
            modifier = Modifier.padding(top = it.calculateTopPadding()),
            uiState.animateMap,
            uiState.location?.toLatLng() ?: gothenburgLatLng,
            marker = uiState.tunnelRealState.toMarker(uiState.location),
            bias,
        )
        ConstraintLayout(
            modifier = Modifier.padding(top = it.calculateTopPadding()).fillMaxSize()
        ) {
            val (divider) = createRefs()
            Divider(
                modifier =
                    Modifier.background(Color.Cyan).constrainAs(divider) {
                        start.linkTo(parent.start)
                        end.linkTo(parent.end)
                        top.linkTo(parent.top)
                        centerVerticallyTo(parent, bias)
                    }
            )
        }
        Column(
            verticalArrangement = Arrangement.Bottom,
            horizontalAlignment = Alignment.Start,
            modifier =
                Modifier.padding(it)
                    .fillMaxHeight()
                    .drawVerticalScrollbar(
                        scrollState,
                        color = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaScrollbar)
                    )
                    .verticalScroll(scrollState)
                    .padding(bottom = Dimens.screenVerticalMargin)
                    .testTag(SCROLLABLE_COLUMN_TEST_TAG)
        ) {
            NotificationBanner(
                notification = uiState.inAppNotification,
                isPlayBuild = uiState.isPlayBuild,
                onClickUpdateVersion = onUpdateVersionClick,
                onClickShowAccount = onManageAccountClick,
                onClickDismissNewDevice = onDismissNewDeviceClick,
            )
            Slider(
                modifier = Modifier.padding(16.dp),
                value = bias,
                onValueChange = { bias = it },
                valueRange = 0f..1f,
            )
            Text(text = "Bias: $bias")
            Spacer(modifier = Modifier.weight(1f))
            if (
                uiState.tunnelRealState is TunnelState.Connecting ||
                    uiState.tunnelRealState is TunnelState.Disconnecting
            ) {
                MullvadCircularProgressIndicatorLarge(
                    color = MaterialTheme.colorScheme.onPrimary,
                    modifier =
                        Modifier.padding(
                                start = Dimens.sideMargin,
                                end = Dimens.sideMargin,
                                top = Dimens.mediumPadding
                            )
                            .align(Alignment.CenterHorizontally)
                            .testTag(CIRCULAR_PROGRESS_INDICATOR)
                )
            }
            Spacer(modifier = Modifier.height(Dimens.mediumPadding))
            ConnectionStatusText(
                state = uiState.tunnelRealState,
                modifier = Modifier.padding(horizontal = Dimens.sideMargin)
            )
            Text(
                text = uiState.location?.country ?: "",
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.onPrimary,
                modifier = Modifier.padding(horizontal = Dimens.sideMargin)
            )
            Text(
                text = uiState.location?.city ?: "",
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.onPrimary,
                modifier = Modifier.padding(horizontal = Dimens.sideMargin)
            )
            var expanded by rememberSaveable { mutableStateOf(false) }
            LocationInfo(
                onToggleTunnelInfo = { expanded = !expanded },
                isVisible =
                    uiState.tunnelRealState !is TunnelState.Disconnected &&
                        uiState.location?.hostname != null,
                isExpanded = expanded,
                location = uiState.location,
                inAddress = uiState.inAddress,
                outAddress = uiState.outAddress,
                modifier =
                    Modifier.fillMaxWidth()
                        .padding(horizontal = Dimens.sideMargin)
                        .testTag(LOCATION_INFO_TEST_TAG)
            )
            Spacer(modifier = Modifier.height(Dimens.buttonSpacing))
            SwitchLocationButton(
                modifier =
                    Modifier.fillMaxWidth()
                        .padding(horizontal = Dimens.sideMargin)
                        .testTag(SELECT_LOCATION_BUTTON_TEST_TAG),
                onClick = onSwitchLocationClick,
                showChevron = uiState.showLocation,
                text =
                    if (uiState.showLocation) {
                        uiState.relayLocation?.locationName ?: ""
                    } else {
                        stringResource(id = R.string.switch_location)
                    }
            )
            Spacer(modifier = Modifier.height(Dimens.buttonSpacing))
            ConnectionButton(
                state = uiState.tunnelUiState,
                modifier =
                    Modifier.padding(horizontal = Dimens.sideMargin)
                        .testTag(CONNECT_BUTTON_TEST_TAG),
                disconnectClick = onDisconnectClick,
                reconnectClick = { handleThrottledAction(onReconnectClick) },
                cancelClick = onCancelClick,
                connectClick = { handleThrottledAction(onConnectClick) },
                reconnectButtonTestTag = RECONNECT_BUTTON_TEST_TAG
            )
        }
    }
}

fun TunnelState.toMarker(location: GeoIpLocation?): Marker? {
    if (location == null) return null
    return when (this) {
        is TunnelState.Connected -> Marker(location.toLatLng(), MarkerType.SECURE)
        is TunnelState.Connecting -> null
        is TunnelState.Disconnected -> Marker(location.toLatLng(), MarkerType.UNSECURE)
        is TunnelState.Disconnecting -> null
        is TunnelState.Error -> null
    }
}

fun GeoIpLocation.toLatLng() = LatLng(latitude.toFloat(), longitude.toFloat())

package net.mullvad.mullvadvpn.compose.screen

import android.content.Intent
import android.net.Uri
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInParent
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.popUpTo
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
import net.mullvad.mullvadvpn.constant.SECURE_ZOOM
import net.mullvad.mullvadvpn.constant.SECURE_ZOOM_ANIMATION_MILLIS
import net.mullvad.mullvadvpn.constant.UNSECURE_ZOOM
import net.mullvad.mullvadvpn.constant.fallbackLatLong
import net.mullvad.mullvadvpn.lib.common.util.openAccountPageInBrowser
import net.mullvad.mullvadvpn.lib.map.AnimatedMap
import net.mullvad.mullvadvpn.lib.map.data.GlobeColors
import net.mullvad.mullvadvpn.lib.map.data.LocationMarkerColors
import net.mullvad.mullvadvpn.lib.map.data.Marker
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInvisible
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.theme.color.AlphaTopBar
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.LatLong
import net.mullvad.mullvadvpn.model.Latitude
import net.mullvad.mullvadvpn.model.Longitude
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
            state = state,
        )
    }
}

@Destination(style = HomeTransition::class)
@Composable
fun Connect(navigator: DestinationsNavigator) {
    val connectViewModel: ConnectViewModel = koinViewModel()

    val state by connectViewModel.uiState.collectAsStateWithLifecycle()

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
        state = state,
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
private fun ConnectScreen(
    state: ConnectUiState,
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

    ScaffoldWithTopBarAndDeviceName(
        topBarColor = state.tunnelUiState.topBarColor(),
        iconTintColor = state.tunnelUiState.iconTintColor(),
        onSettingsClicked = onSettingsClick,
        onAccountClicked = onAccountClick,
        deviceName = state.deviceName,
        timeLeft = state.daysLeftUntilExpiry
    ) {
        var progressIndicatorBias by remember { mutableFloatStateOf(0f) }

        MapColumn(
            state,
            it,
            progressIndicatorBias,
            scrollState,
        ) {
            Spacer(modifier = Modifier.defaultMinSize(minHeight = Dimens.mediumPadding).weight(1f))
            MullvadCircularProgressIndicatorLarge(
                color = MaterialTheme.colorScheme.onPrimary,
                modifier =
                    Modifier.animateContentSize()
                        .padding(
                            start = Dimens.sideMargin,
                            end = Dimens.sideMargin,
                            top = Dimens.mediumPadding
                        )
                        .alpha(if (state.showLoading) AlphaVisible else AlphaInvisible)
                        .align(Alignment.CenterHorizontally)
                        .testTag(CIRCULAR_PROGRESS_INDICATOR)
                        .onGloballyPositioned {
                            val offsetY = it.positionInParent().y + it.size.height / 2
                            it.parentLayoutCoordinates?.let {
                                val parentHeight = it.size.height
                                val verticalBias = offsetY / parentHeight
                                if (verticalBias.isFinite()) {
                                    progressIndicatorBias = verticalBias
                                }
                            }
                        }
            )
            Spacer(modifier = Modifier.defaultMinSize(minHeight = Dimens.mediumPadding).weight(1f))

            ConnectionInfo(state = state)

            Spacer(modifier = Modifier.height(Dimens.buttonSpacing))

            ButtonPanel(
                state,
                onSwitchLocationClick,
                onDisconnectClick,
                onReconnectClick,
                onCancelClick,
                onConnectClick,
            )
        }

        NotificationBanner(
            modifier = Modifier.padding(top = it.calculateTopPadding()),
            notification = state.inAppNotification,
            isPlayBuild = state.isPlayBuild,
            onClickUpdateVersion = onUpdateVersionClick,
            onClickShowAccount = onManageAccountClick,
            onClickDismissNewDevice = onDismissNewDeviceClick,
        )
    }
}

@Composable
private fun MapColumn(
    state: ConnectUiState,
    it: PaddingValues,
    progressIndicatorBias: Float,
    scrollState: ScrollState,
    content: @Composable ColumnScope.() -> Unit
) {

    // Distance to marker when secure/unsecure
    val baseZoom =
        animateFloatAsState(
            targetValue =
                if (state.tunnelRealState is TunnelState.Connected) SECURE_ZOOM else UNSECURE_ZOOM,
            animationSpec = tween(SECURE_ZOOM_ANIMATION_MILLIS),
            label = "baseZoom"
        )

    val markers = state.tunnelRealState.toMarker(state.location)?.let { listOf(it) } ?: emptyList()

    AnimatedMap(
        modifier = Modifier.padding(top = it.calculateTopPadding()),
        cameraLocation = state.location?.toLatLong() ?: fallbackLatLong,
        cameraBaseZoom = baseZoom.value,
        cameraVerticalBias = progressIndicatorBias,
        markers = markers,
        globeColors =
            GlobeColors(
                landColor = MaterialTheme.colorScheme.primary,
                oceanColor = MaterialTheme.colorScheme.secondary,
            )
    )

    Column(
        verticalArrangement = Arrangement.Bottom,
        horizontalAlignment = Alignment.Start,
        modifier =
            Modifier.animateContentSize()
                .padding(top = it.calculateTopPadding())
                .fillMaxHeight()
                .drawVerticalScrollbar(
                    scrollState,
                    color = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaScrollbar)
                )
                .verticalScroll(scrollState)
                .testTag(SCROLLABLE_COLUMN_TEST_TAG)
    ) {
        content()
        // We need to manually add this padding so we align size with the map
        // component and marker with the progress indicator.
        Spacer(modifier = Modifier.height(it.calculateBottomPadding()))
    }
}

@Composable
private fun ConnectionInfo(state: ConnectUiState) {
    ConnectionStatusText(
        state = state.tunnelRealState,
        modifier = Modifier.padding(horizontal = Dimens.sideMargin)
    )
    Text(
        text = state.location?.country ?: "",
        style = MaterialTheme.typography.headlineLarge,
        color = MaterialTheme.colorScheme.onPrimary,
        modifier = Modifier.padding(horizontal = Dimens.sideMargin)
    )
    Text(
        text = state.location?.city ?: "",
        style = MaterialTheme.typography.headlineLarge,
        color = MaterialTheme.colorScheme.onPrimary,
        modifier = Modifier.padding(horizontal = Dimens.sideMargin)
    )
    var expanded by rememberSaveable { mutableStateOf(false) }
    LocationInfo(
        onToggleTunnelInfo = { expanded = !expanded },
        isVisible = state.showLocationInfo,
        isExpanded = expanded,
        location = state.location,
        inAddress = state.inAddress,
        outAddress = state.outAddress,
        modifier =
            Modifier.fillMaxWidth()
                .padding(horizontal = Dimens.sideMargin)
                .testTag(LOCATION_INFO_TEST_TAG)
    )
}

@Composable
private fun ButtonPanel(
    state: ConnectUiState,
    onSwitchLocationClick: () -> Unit,
    onDisconnectClick: () -> Unit,
    onReconnectClick: () -> Unit,
    onCancelClick: () -> Unit,
    onConnectClick: () -> Unit,
) {
    var lastConnectionActionTimestamp by remember { mutableLongStateOf(0L) }

    fun handleThrottledAction(action: () -> Unit) {
        val currentTime = System.currentTimeMillis()
        if ((currentTime - lastConnectionActionTimestamp) > CONNECT_BUTTON_THROTTLE_MILLIS) {
            lastConnectionActionTimestamp = currentTime
            action.invoke()
        }
    }

    SwitchLocationButton(
        modifier =
            Modifier.fillMaxWidth()
                .padding(horizontal = Dimens.sideMargin)
                .testTag(SELECT_LOCATION_BUTTON_TEST_TAG),
        onClick = onSwitchLocationClick,
        showChevron = state.showLocation,
        text =
            if (state.showLocation && state.selectedRelayItem != null) {
                state.selectedRelayItem.locationName
            } else {
                stringResource(id = R.string.switch_location)
            }
    )
    Spacer(modifier = Modifier.height(Dimens.buttonSpacing))
    ConnectionButton(
        state = state.tunnelUiState,
        modifier =
            Modifier.padding(horizontal = Dimens.sideMargin)
                .padding(bottom = Dimens.screenVerticalMargin)
                .testTag(CONNECT_BUTTON_TEST_TAG),
        disconnectClick = onDisconnectClick,
        reconnectClick = { handleThrottledAction(onReconnectClick) },
        cancelClick = onCancelClick,
        connectClick = { handleThrottledAction(onConnectClick) },
        reconnectButtonTestTag = RECONNECT_BUTTON_TEST_TAG
    )
}

@Composable
fun TunnelState.toMarker(location: GeoIpLocation?): Marker? {
    if (location == null) return null
    return when (this) {
        is TunnelState.Connected ->
            Marker(
                location.toLatLong(),
                colors =
                    LocationMarkerColors(centerColor = MaterialTheme.colorScheme.inversePrimary),
            )
        is TunnelState.Connecting -> null
        is TunnelState.Disconnected ->
            Marker(
                location.toLatLong(),
                colors = LocationMarkerColors(centerColor = MaterialTheme.colorScheme.error)
            )
        is TunnelState.Disconnecting -> null
        is TunnelState.Error -> null
    }
}

@Composable
fun TunnelState.topBarColor(): Color =
    if (isSecured()) MaterialTheme.colorScheme.inversePrimary else MaterialTheme.colorScheme.error

@Composable
fun TunnelState.iconTintColor(): Color =
    if (isSecured()) {
            MaterialTheme.colorScheme.onPrimary
        } else {
            MaterialTheme.colorScheme.onError
        }
        .copy(alpha = AlphaTopBar)

fun GeoIpLocation.toLatLong() =
    LatLong(Latitude(latitude.toFloat()), Longitude(longitude.toFloat()))

package net.mullvad.mullvadvpn.widget

import android.content.Context
import android.content.Intent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.glance.Button
import androidx.glance.GlanceId
import androidx.glance.GlanceModifier
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.SizeMode
import androidx.glance.appwidget.action.actionStartService
import androidx.glance.appwidget.provideContent
import androidx.glance.appwidget.updateAll
import androidx.glance.background
import androidx.glance.layout.Alignment
import androidx.glance.layout.Column
import androidx.glance.layout.fillMaxSize
import androidx.glance.text.Text
import androidx.glance.text.TextDefaults.defaultTextStyle
import androidx.glance.text.TextStyle
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.common.constant.KEY_CONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.KEY_DISCONNECT_ACTION
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import org.koin.core.context.GlobalContext

class MullvadAppWidget : GlanceAppWidget() {

    private lateinit var connectionProxy: ConnectionProxy

    override val sizeMode = SizeMode.Responsive(
        setOf(
            SHORT_NARROW_SIZE, SHORT_SLIM_SIZE, SHORT_WIDE_SIZE,
            MEDIUM_NARROW_SIZE, MEDIUM_WIDE_SIZE,
            TALL_WIDE_SIZE, TALL_XWIDE_SIZE,
            XTALL_WIDE_SIZE, XTALL_XWIDE_SIZE
        )
    )

    override suspend fun provideGlance(context: Context, id: GlanceId) {
        with(GlobalContext.get()) { connectionProxy = get<ConnectionProxy>() }
        val widgetProvider = WidgetProvider(connectionProxy)
        val packageName = context.packageName
        provideContent {
            val state = widgetProvider.state().collectAsState().value
            Widget(state = state, packageName = packageName)
        }
    }

    @Composable
    private fun Widget(state: TunnelState = TunnelState.Disconnected(), packageName: String) {
        Column(
            modifier = GlanceModifier.fillMaxSize().background(Color.White),
            verticalAlignment = Alignment.CenterVertically,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Logger.i("State: $state")
            val textStyle = defaultTextStyle.copy(
                fontSize = 24.sp,
            )
            Text(text = "Mullvad VPN", style = textStyle)
            Text(text = "Status: ${state.displayState()}", style = textStyle)
            Text(text = "Location: ${state.location()?.country ?: "Unknown"}", style = textStyle)
            Button(
                text = state.buttonState(),
                onClick =
                    actionStartService(
                        intent =
                            Intent().apply {
                                setClassName(packageName, VPN_SERVICE_CLASS)
                                action = state.mapToAction()
                            },
                        isForegroundService = true,
                    ),
                style = textStyle,
            )
        }
    }

    private fun TunnelState.displayState() =
        when (this) {
            is TunnelState.Connected -> "Connected"
            is TunnelState.Connecting -> "Connecting"
            is TunnelState.Disconnected -> "Disconnected"
            is TunnelState.Disconnecting -> "Disconnecting"
            is TunnelState.Error -> "Error"
        }

    private fun TunnelState.buttonState() =
        when (this) {
            is TunnelState.Connected -> "Disconnect"
            is TunnelState.Connecting -> "Cancel"
            is TunnelState.Disconnected -> "Connect"
            is TunnelState.Disconnecting -> "Disconnect"
            is TunnelState.Error -> "Cancel"
        }

    private fun TunnelState.mapToAction(): String =
        when (this) {
            is TunnelState.Disconnected -> KEY_CONNECT_ACTION
            is TunnelState.Connecting -> KEY_DISCONNECT_ACTION
            is TunnelState.Connected -> KEY_DISCONNECT_ACTION
            is TunnelState.Disconnecting -> {
                if (actionAfterDisconnect == ActionAfterDisconnect.Reconnect) {
                    KEY_DISCONNECT_ACTION
                } else {
                    KEY_CONNECT_ACTION
                }
            }
            is TunnelState.Error -> {
                if (errorState.isBlocking) {
                    KEY_DISCONNECT_ACTION
                } else {
                    KEY_CONNECT_ACTION
                }
            }
        }

    companion object {

        private val SHORT = 149.dp
        private val MEDIUM = 200.dp
        private val TALL = 331.dp
        private val XTALL = 426.dp

        private val NARROW = 100.dp
        private val SLIM = 140.dp
        private val WIDE = 210.dp
        private val XWIDE = 280.dp

        private val SHORT_NARROW_SIZE = DpSize(NARROW, SHORT)
        private val SHORT_SLIM_SIZE = DpSize(SLIM, SHORT)
        private val SHORT_WIDE_SIZE = DpSize(WIDE, SHORT)

        private val MEDIUM_NARROW_SIZE = DpSize(NARROW, MEDIUM)
        private val MEDIUM_WIDE_SIZE = DpSize(WIDE, MEDIUM)

        private val TALL_WIDE_SIZE = DpSize(WIDE, TALL)
        private val TALL_XWIDE_SIZE = DpSize(XWIDE, TALL)

        private val XTALL_WIDE_SIZE = DpSize(WIDE, XTALL)
        private val XTALL_XWIDE_SIZE = DpSize(XWIDE, XTALL)

        suspend fun updateAllWidgets(context: Context) {
            MullvadAppWidget().updateAll(context)
        }
    }
}

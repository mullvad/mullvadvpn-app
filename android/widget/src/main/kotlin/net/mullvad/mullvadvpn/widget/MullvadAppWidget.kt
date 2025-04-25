package net.mullvad.mullvadvpn.widget

import android.appwidget.AppWidgetManager
import android.content.Context
import android.content.Intent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.stringSetPreferencesKey
import androidx.glance.ButtonDefaults
import androidx.glance.GlanceId
import androidx.glance.GlanceModifier
import androidx.glance.GlanceTheme
import androidx.glance.Image
import androidx.glance.ImageProvider
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.GlanceAppWidgetManager
import androidx.glance.appwidget.SizeMode
import androidx.glance.appwidget.Switch
import androidx.glance.appwidget.SwitchDefaults
import androidx.glance.appwidget.action.actionSendBroadcast
import androidx.glance.appwidget.components.FilledButton
import androidx.glance.appwidget.components.Scaffold
import androidx.glance.appwidget.provideContent
import androidx.glance.appwidget.state.getAppWidgetState
import androidx.glance.appwidget.state.updateAppWidgetState
import androidx.glance.appwidget.updateAll
import androidx.glance.currentState
import androidx.glance.layout.Alignment
import androidx.glance.layout.Column
import androidx.glance.layout.Row
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.height
import androidx.glance.layout.padding
import androidx.glance.layout.wrapContentHeight
import androidx.glance.text.Text
import androidx.glance.text.TextStyle
import androidx.glance.unit.ColorProvider
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.common.constant.KEY_UPDATE_SETTING
import net.mullvad.mullvadvpn.lib.common.constant.WIDGET_ACTION_RECEIVER
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.shared.WidgetRepository
import net.mullvad.mullvadvpn.lib.shared.WidgetSettingsState
import net.mullvad.mullvadvpn.lib.shared.WidgetSettingsState.Companion.PREF_KEY
import org.koin.core.context.GlobalContext

class MullvadAppWidget : GlanceAppWidget() {

    private lateinit var widgetRepository: WidgetRepository

    override val sizeMode = SizeMode.Single

    override suspend fun provideGlance(context: Context, id: GlanceId) {
        val manager = GlanceAppWidgetManager(context)
        val info =
            AppWidgetManager.getInstance(context).getAppWidgetInfo(manager.getAppWidgetId(id))
        with(GlobalContext.get()) { widgetRepository = get<WidgetRepository>() }
        val widgetProvider = WidgetProvider(widgetRepository)
        val packageName = context.packageName
        provideContent {
            val state = widgetProvider.settings().collectAsState().value
            val prefs = currentState<Set<String>>(stringSetPreferencesKey(PREF_KEY))
            Widget(
                settings = state,
                widgetSettings = WidgetSettingsState.fromPrefs(prefs ?: emptySet()),
                type = WidgetType.fromClass(info.provider.shortClassName),
                packageName = packageName,
            )
        }
    }

    @Composable
    private fun Widget(
        settings: Settings?,
        widgetSettings: WidgetSettingsState,
        type: WidgetType,
        packageName: String,
    ) {
        WidgetTheme {
            when (type) {
                WidgetType.SETTINGS ->
                    SettingsWidget(
                        widgetSettingsState = widgetSettings,
                        allowLan = settings.allowLan(),
                        customDnsEnabled = settings.customDns(),
                        customDnsAvailable = !settings.isAnyDnsBlockerEnabled(),
                        daitaEnabled = settings.daitaEnabled(),
                        splitTunnelingEnabled = settings.splitTunnelingEnabled(),
                        multihopEnabled = settings.multihopEnabled(),
                        inTunnelIpv6Enabled = settings.inTunnelIpv6Enabled(),
                        quantumResistantState = settings.quantumResistantState(),
                        packageName = packageName,
                    )
            }
        }
    }

    @Composable
    private fun SettingsWidget(
        widgetSettingsState: WidgetSettingsState,
        allowLan: Boolean,
        customDnsEnabled: Boolean,
        customDnsAvailable: Boolean,
        daitaEnabled: Boolean,
        splitTunnelingEnabled: Boolean,
        multihopEnabled: Boolean,
        inTunnelIpv6Enabled: Boolean,
        quantumResistantState: QuantumResistantState,
        packageName: String,
    ) {
        Scaffold(
            backgroundColor = GlanceTheme.colors.widgetBackground,//ColorProvider(Color(0xFF254669)),
            titleBar = {
                Row(
                    modifier = GlanceModifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Image(
                        modifier = GlanceModifier.height(32.dp),
                        provider = ImageProvider(R.drawable.logo_icon),
                        contentDescription = null, // No meaningful user info or action.
                    )
                    Text(
                        text = "Mullvad Settings",
                        modifier = GlanceModifier.padding(16.dp),
                        style = TextStyle(color = ColorProvider(Color.White)),
                    )
                }
            },
        ) {
            Column(
                modifier = GlanceModifier.fillMaxWidth().wrapContentHeight(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Logger.i("allowLan: $allowLan")
                Logger.i("customDnsEnabled: $customDnsEnabled")
                Logger.i("daitaEnabled: $daitaEnabled")
                Logger.i("splitTunnelingEnabled: $splitTunnelingEnabled")
                val textStyle = TextStyle(color = ColorProvider(Color.White))
                val switchColors =
                    SwitchDefaults.colors(
                        checkedThumbColor = ColorProvider(Color(0xFF44AD4D)),
                        checkedTrackColor = ColorProvider(Color(0xFF44AD4D)),
                        uncheckedThumbColor = ColorProvider(Color(0xFFE34039)),
                        uncheckedTrackColor = ColorProvider(Color(0xFFE34039)),
                    )
                if (widgetSettingsState.showLan) {
                    Row(modifier = GlanceModifier.fillMaxWidth()) {
                        Text("Local Network Sharing", style = textStyle)
                        Spacer(modifier = GlanceModifier.defaultWeight())
                        Switch(
                            checked = allowLan,
                            onCheckedChange =
                                actionSendBroadcast(
                                    Intent().apply {
                                        setClassName(packageName, WIDGET_ACTION_RECEIVER)
                                        action = KEY_UPDATE_SETTING
                                        putExtra("SETTING", "LAN")
                                    }
                                ),
                            colors = switchColors,
                        )
                    }
                }
                if (widgetSettingsState.showCustomDns) {
                    Row(modifier = GlanceModifier.fillMaxWidth()) {
                        Text(
                            "Custom DNS Server",
                            style = textStyle.copy(color = ColorProvider(Color.Gray)),
                        )
                        Spacer(modifier = GlanceModifier.defaultWeight())
                        if (customDnsAvailable) {
                            Switch(
                                checked = customDnsEnabled,
                                onCheckedChange =
                                    actionSendBroadcast(
                                        Intent().apply {
                                            setClassName(packageName, WIDGET_ACTION_RECEIVER)
                                            action = KEY_UPDATE_SETTING
                                            putExtra("SETTING", "CUSTOM_DNS")
                                        }
                                    ),
                                colors = switchColors,
                            )
                        }
                    }
                }
                if (widgetSettingsState.showDaita) {
                    Row(modifier = GlanceModifier.fillMaxWidth()) {
                        Text("DAITA", style = textStyle)
                        Spacer(modifier = GlanceModifier.defaultWeight())
                        Switch(
                            checked = daitaEnabled,
                            onCheckedChange =
                                actionSendBroadcast(
                                    Intent().apply {
                                        setClassName(packageName, WIDGET_ACTION_RECEIVER)
                                        action = KEY_UPDATE_SETTING
                                        putExtra("SETTING", "DAITA")
                                    }
                                ),
                            colors = switchColors,
                        )
                    }
                }
                if (widgetSettingsState.showSplitTunneling) {
                    Row(modifier = GlanceModifier.fillMaxWidth()) {
                        Text("Split Tunneling", style = textStyle)
                        Spacer(modifier = GlanceModifier.defaultWeight())
                        Switch(
                            checked = splitTunnelingEnabled,
                            onCheckedChange =
                                actionSendBroadcast(
                                    Intent().apply {
                                        setClassName(packageName, WIDGET_ACTION_RECEIVER)
                                        action = KEY_UPDATE_SETTING
                                        putExtra("SETTING", "SPLIT_TUNNELING")
                                    }
                                ),
                            colors = switchColors,
                        )
                    }
                }
                if (widgetSettingsState.showMultihop) {
                    Row(modifier = GlanceModifier.fillMaxWidth()) {
                        Text("Multihop", style = textStyle)
                        Spacer(modifier = GlanceModifier.defaultWeight())
                        Switch(
                            checked = multihopEnabled,
                            onCheckedChange =
                                actionSendBroadcast(
                                    Intent().apply {
                                        setClassName(packageName, WIDGET_ACTION_RECEIVER)
                                        action = KEY_UPDATE_SETTING
                                        putExtra("SETTING", "MULITIHOP")
                                    }
                                ),
                            colors = switchColors,
                        )
                    }
                }
                if (widgetSettingsState.showInTunnelIpv6) {
                    Row(modifier = GlanceModifier.fillMaxWidth()) {
                        Text("In-tunnel IPv6", style = textStyle)
                        Spacer(modifier = GlanceModifier.defaultWeight())
                        Switch(
                            checked = inTunnelIpv6Enabled,
                            onCheckedChange =
                                actionSendBroadcast(
                                    Intent().apply {
                                        setClassName(packageName, WIDGET_ACTION_RECEIVER)
                                        action = KEY_UPDATE_SETTING
                                        putExtra("SETTING", "IN_TUNNEL_IPV6")
                                    }
                                ),
                            colors = switchColors,
                        )
                    }
                }
                if (widgetSettingsState.showQuantumResistant) {
                    Row(
                        modifier = GlanceModifier.fillMaxWidth(),
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        Text("Quantum-resistant tunnel", style = textStyle)
                        Spacer(modifier = GlanceModifier.defaultWeight())
                        FilledButton(
                            text =
                                when (quantumResistantState) {
                                    QuantumResistantState.Auto -> "Auto"
                                    QuantumResistantState.On -> "On"
                                    QuantumResistantState.Off -> "Off"
                                },
                            onClick =
                                actionSendBroadcast(
                                    Intent().apply {
                                        setClassName(packageName, WIDGET_ACTION_RECEIVER)
                                        action = KEY_UPDATE_SETTING
                                        putExtra("SETTING", "QUANTUM_RESISTANT")
                                        putExtra(
                                            "STATE",
                                            when (quantumResistantState) {
                                                QuantumResistantState.Auto -> "On"
                                                QuantumResistantState.On -> "Off"
                                                QuantumResistantState.Off -> "Auto"
                                            },
                                        )
                                    }
                                ),
                            colors =
                                ButtonDefaults.buttonColors(
                                    backgroundColor =
                                        when (quantumResistantState) {
                                            QuantumResistantState.Auto -> ColorProvider(Color.Gray)
                                            QuantumResistantState.On ->
                                                ColorProvider(Color(0xFF44AD4D))
                                            QuantumResistantState.Off ->
                                                ColorProvider(Color(0xFFE34039))
                                        },
                                    contentColor = ColorProvider(Color.White),
                                ),
                        )
                    }
                }
            }
        }
    }

    private fun Settings?.allowLan() = this?.allowLan == true

    private fun Settings?.customDns() = this?.tunnelOptions?.dnsOptions?.state == DnsState.Custom

    private fun Settings?.daitaEnabled() =
        this?.tunnelOptions?.wireguard?.daitaSettings?.enabled == true

    private fun Settings?.splitTunnelingEnabled() = this?.splitTunnelSettings?.enabled == true

    private fun Settings?.multihopEnabled() =
        this?.relaySettings?.relayConstraints?.wireguardConstraints?.isMultihopEnabled == true

    private fun Settings?.inTunnelIpv6Enabled() =
        this?.tunnelOptions?.genericOptions?.enableIpv6 == true

    private fun Settings?.isAnyDnsBlockerEnabled() =
        this?.tunnelOptions?.dnsOptions?.defaultOptions?.isAnyBlockerEnabled() == true

    private fun Settings?.quantumResistantState() =
        this?.tunnelOptions?.wireguard?.quantumResistant ?: QuantumResistantState.Auto

    companion object {
        suspend fun updateAllWidgets(context: Context) {
            MullvadAppWidget().updateAll(context)
        }

        suspend fun updateWidget(context: Context, widgetId: GlanceId) {
            MullvadAppWidget().update(context, widgetId)
        }

        suspend fun updateWidget(context: Context, appWidgetId: Int) {
            val glanceAppWidgetManager = GlanceAppWidgetManager(context)
            val glanceId: GlanceId = glanceAppWidgetManager.getGlanceIdBy(appWidgetId)
            updateWidget(context, glanceId)
        }

        suspend fun getPrefsForWidget(context: Context, appWidgetId: Int): Set<String>? {
            val glanceAppWidgetManager = GlanceAppWidgetManager(context)
            val glanceId: GlanceId = glanceAppWidgetManager.getGlanceIdBy(appWidgetId)
            return MullvadAppWidget()
                .getAppWidgetState<Preferences>(context, glanceId)[
                    stringSetPreferencesKey(PREF_KEY)]
        }

        suspend fun updateWidgetState(
            context: Context,
            appWidgetId: Int,
            widgetSettingsState: WidgetSettingsState,
        ) {
            val glanceAppWidgetManager = GlanceAppWidgetManager(context)
            val glanceId: GlanceId = glanceAppWidgetManager.getGlanceIdBy(appWidgetId)
            updateAppWidgetState(context, glanceId) { prefs ->
                prefs[stringSetPreferencesKey(PREF_KEY)] = widgetSettingsState.toPrefs()
            }
        }
    }
}

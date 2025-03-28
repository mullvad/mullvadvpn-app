package net.mullvad.mullvadvpn.widget

import android.appwidget.AppWidgetManager
import android.content.Context
import android.content.Intent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.glance.GlanceId
import androidx.glance.GlanceModifier
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.GlanceAppWidgetManager
import androidx.glance.appwidget.SizeMode
import androidx.glance.appwidget.Switch
import androidx.glance.appwidget.SwitchDefaults
import androidx.glance.appwidget.action.actionStartService
import androidx.glance.appwidget.provideContent
import androidx.glance.appwidget.updateAll
import androidx.glance.background
import androidx.glance.layout.Alignment
import androidx.glance.layout.Column
import androidx.glance.layout.Row
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxSize
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.padding
import androidx.glance.layout.wrapContentHeight
import androidx.glance.text.Text
import androidx.glance.text.TextStyle
import androidx.glance.unit.ColorProvider
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.lib.common.constant.KEY_SET_BLOCK_DNS
import net.mullvad.mullvadvpn.lib.common.constant.KEY_UPDATE_SETTING
import net.mullvad.mullvadvpn.lib.common.constant.VPN_SERVICE_CLASS
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.shared.WidgetRepository
import net.mullvad.mullvadvpn.lib.shared.WidgetSettingsState
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
            val widgetSettings = widgetProvider.widgetSettings().collectAsState().value
            Widget(
                settings = state,
                widgetSettings = widgetSettings,
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
        when (type) {
            WidgetType.SETTINGS ->
                SettingsWidget(
                    widgetSettingsState = widgetSettings,
                    allowLan = settings.allowLan(),
                    customDnsEnabled = settings.customDns(),
                    daitaEnabled = settings.daitaEnabled(),
                    splitTunnelingEnabled = settings.splitTunnelingEnabled(),
                    packageName = packageName,
                )
            WidgetType.DNS_CONTENT_BLOCKERS ->
                DnsContentBlockersWidget(
                    blockedByCustomDns = settings.customDns(),
                    dnsOptions = settings.dnsContentBlockers(),
                    packageName = packageName,
                )
        }
    }

    @Composable
    private fun SettingsWidget(
        widgetSettingsState: WidgetSettingsState,
        allowLan: Boolean,
        customDnsEnabled: Boolean,
        daitaEnabled: Boolean,
        splitTunnelingEnabled: Boolean,
        packageName: String,
    ) {
        rememberAppWidgetConfigurationState
        Column(
            modifier =
                GlanceModifier.fillMaxWidth()
                    .wrapContentHeight()
                    .padding(16.dp)
                    .background(Color(0xFF254669)),
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
                            actionStartService(
                                Intent().apply {
                                    setClassName(packageName, VPN_SERVICE_CLASS)
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
                    Text("Custom DNS Server", style = textStyle)
                    Spacer(modifier = GlanceModifier.defaultWeight())
                    Switch(
                        checked = customDnsEnabled,
                        onCheckedChange =
                            actionStartService(
                                Intent().apply {
                                    setClassName(packageName, VPN_SERVICE_CLASS)
                                    action = KEY_UPDATE_SETTING
                                    putExtra("SETTING", "CUSTOM_DNS")
                                }
                            ),
                        colors = switchColors,
                    )
                }
            }
            if (widgetSettingsState.showDaita) {
                Row(modifier = GlanceModifier.fillMaxWidth()) {
                    Text("DAITA", style = textStyle)
                    Spacer(modifier = GlanceModifier.defaultWeight())
                    Switch(
                        checked = daitaEnabled,
                        onCheckedChange =
                            actionStartService(
                                Intent().apply {
                                    setClassName(packageName, VPN_SERVICE_CLASS)
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
                            actionStartService(
                                Intent().apply {
                                    setClassName(packageName, VPN_SERVICE_CLASS)
                                    action = KEY_UPDATE_SETTING
                                    putExtra("SETTING", "SPLIT_TUNNELING")
                                }
                            ),
                        colors = switchColors,
                    )
                }
            }
        }
        AppWidgetConfigurationScaffold
    }

    @Composable
    private fun DnsContentBlockersWidget(
        dnsOptions: DefaultDnsOptions,
        blockedByCustomDns: Boolean,
        packageName: String,
    ) {
        Column(
            modifier =
                GlanceModifier.fillMaxSize()
                    .padding(horizontal = 16.dp)
                    .background(Color(0xFF254669)),
            verticalAlignment = Alignment.CenterVertically,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Logger.i("DnsOptions: $dnsOptions")
            val textStyle = TextStyle(color = ColorProvider(Color.White))
            val switchColors =
                SwitchDefaults.colors(
                    checkedThumbColor = ColorProvider(Color(0xFF44AD4D)),
                    checkedTrackColor = ColorProvider(Color(0xFF44AD4D)),
                    uncheckedThumbColor = ColorProvider(Color(0xFFE34039)),
                    uncheckedTrackColor = ColorProvider(Color(0xFFE34039)),
                )
            Row(modifier = GlanceModifier.fillMaxWidth()) {
                Text("Block Ads", style = textStyle)
                Spacer(modifier = GlanceModifier.defaultWeight())
                Switch(
                    checked = dnsOptions.blockAds,
                    onCheckedChange =
                        actionStartService(
                            Intent().apply {
                                setClassName(packageName, VPN_SERVICE_CLASS)
                                action = KEY_SET_BLOCK_DNS
                                putExtra("ACTION", "BLOCK_ADS")
                            }
                        ),
                    colors = switchColors,
                )
            }
            Row(modifier = GlanceModifier.fillMaxWidth()) {
                Text("Block Trackers", style = textStyle)
                Spacer(modifier = GlanceModifier.defaultWeight())
                Switch(
                    checked = dnsOptions.blockTrackers,
                    onCheckedChange =
                        actionStartService(
                            Intent().apply {
                                setClassName(packageName, VPN_SERVICE_CLASS)
                                action = KEY_SET_BLOCK_DNS
                                putExtra("ACTION", "BLOCK_TRACKERS")
                            }
                        ),
                    colors = switchColors,
                )
            }
            Row(modifier = GlanceModifier.fillMaxWidth()) {
                Text("Block Malware", style = textStyle)
                Spacer(modifier = GlanceModifier.defaultWeight())
                Switch(
                    checked = dnsOptions.blockMalware,
                    onCheckedChange =
                        actionStartService(
                            Intent().apply {
                                setClassName(packageName, VPN_SERVICE_CLASS)
                                action = KEY_SET_BLOCK_DNS
                                putExtra("ACTION", "BLOCK_MALWARE")
                            }
                        ),
                    colors = switchColors,
                )
            }
            Row(modifier = GlanceModifier.fillMaxWidth()) {
                Text("Block Adult content", style = textStyle)
                Spacer(modifier = GlanceModifier.defaultWeight())
                Switch(
                    checked = dnsOptions.blockAdultContent,
                    onCheckedChange =
                        actionStartService(
                            Intent().apply {
                                setClassName(packageName, VPN_SERVICE_CLASS)
                                action = KEY_SET_BLOCK_DNS
                                putExtra("ACTION", "BLOCK_ADULT_CONTENT")
                            }
                        ),
                    colors = switchColors,
                )
            }
            Row(modifier = GlanceModifier.fillMaxWidth()) {
                Text("Block Gambling", style = textStyle)
                Spacer(modifier = GlanceModifier.defaultWeight())
                Switch(
                    checked = dnsOptions.blockGambling,
                    onCheckedChange =
                        actionStartService(
                            Intent().apply {
                                setClassName(packageName, VPN_SERVICE_CLASS)
                                action = KEY_SET_BLOCK_DNS
                                putExtra("ACTION", "BLOCK_GAMBLING")
                            }
                        ),
                    colors = switchColors,
                )
            }
            Row(modifier = GlanceModifier.fillMaxWidth()) {
                Text("Block Social Media", style = textStyle)
                Spacer(modifier = GlanceModifier.defaultWeight())
                Switch(
                    checked = dnsOptions.blockSocialMedia,
                    onCheckedChange =
                        actionStartService(
                            Intent().apply {
                                setClassName(packageName, VPN_SERVICE_CLASS)
                                action = KEY_SET_BLOCK_DNS
                                putExtra("ACTION", "BLOCK_SOCIAL_MEDIA")
                            }
                        ),
                    colors = switchColors,
                )
            }

            if (blockedByCustomDns) {
                Text("Disable CustomDns to activate these settings", style = textStyle)
            }
        }
    }

    private fun Settings?.dnsContentBlockers() =
        this?.tunnelOptions?.dnsOptions?.defaultOptions ?: DefaultDnsOptions()

    private fun Settings?.allowLan() = this?.allowLan == true

    private fun Settings?.customDns() = this?.tunnelOptions?.dnsOptions?.state == DnsState.Custom

    private fun Settings?.daitaEnabled() =
        this?.tunnelOptions?.wireguard?.daitaSettings?.enabled == true

    private fun Settings?.splitTunnelingEnabled() = this?.splitTunnelSettings?.enabled == true

    companion object {
        suspend fun updateAllWidgets(context: Context) {
            MullvadAppWidget().updateAll(context)
        }
    }
}

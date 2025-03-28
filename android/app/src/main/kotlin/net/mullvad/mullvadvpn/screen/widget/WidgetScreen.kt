package net.mullvad.mullvadvpn.screen.widget

import android.app.Activity.RESULT_CANCELED
import android.appwidget.AppWidgetManager
import android.content.Intent
import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.itemWithDivider
import net.mullvad.mullvadvpn.lib.repository.WidgetSettingsPersister
import net.mullvad.mullvadvpn.lib.repository.WidgetSettingsState
import net.mullvad.mullvadvpn.lib.ui.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.InfoListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton

@Composable
fun Widget() {
    val activity = LocalActivity.current
    val widgetSettingsPersister = WidgetSettingsPersister.SINGLETON
    val state by widgetSettingsPersister.widgetSettingsState.collectAsStateWithLifecycle()
    WidgetScreen(
        state = state,
        onBackClick = dropUnlessResumed { activity?.finish() },
        onToggleShowLan = { widgetSettingsPersister.setShowLan(it) },
        onToggleShowCustomDns = { widgetSettingsPersister.setShowCustomDns(it) },
        onToggleShowDaita = { widgetSettingsPersister.setShowDaita(it) },
        onToggleShowSplitTunneling = { widgetSettingsPersister.setShowSplitTunneling(it) },
    )
}

@Composable
fun WidgetScreen(
    state: WidgetSettingsState,
    onBackClick: () -> Unit,
    onToggleShowLan: (Boolean) -> Unit,
    onToggleShowCustomDns: (Boolean) -> Unit,
    onToggleShowDaita: (Boolean) -> Unit,
    onToggleShowSplitTunneling: (Boolean) -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = "Widget Settings",
        navigationIcon = { NavigateCloseIconButton(onBackClick) },
    ) { modifier: Modifier, lazyListState: LazyListState ->
        LazyColumn(modifier = modifier, state = lazyListState) {
            itemWithDivider { InfoListItem(title = "Show settings in widget") }
            itemWithDivider {
                SwitchListItem(
                    title = "Show LAN setting",
                    isToggled = state.showLan,
                    onCellClicked = { onToggleShowLan(it) },
                )
            }
            itemWithDivider {
                SwitchListItem(
                    title = "Show Custom Dns Setting",
                    isToggled = state.showCustomDns,
                    onCellClicked = { onToggleShowCustomDns(it) },
                )
            }
            itemWithDivider {
                SwitchListItem(
                    title = "Show Daita Setting",
                    isToggled = state.showDaita,
                    onCellClicked = { onToggleShowDaita(it) },
                )
            }
            itemWithDivider {
                SwitchListItem(
                    title = "Show Split Tunneling Setting",
                    isToggled = state.showSplitTunneling,
                    onCellClicked = { onToggleShowSplitTunneling(it) },
                )
            }
            item {
                val activity = LocalActivity.current
                PrimaryButton(
                    onClick = {
                        val appWidgetId =
                            activity
                                ?.intent
                                ?.extras
                                ?.getInt(
                                    AppWidgetManager.EXTRA_APPWIDGET_ID,
                                    AppWidgetManager.INVALID_APPWIDGET_ID,
                                ) ?: AppWidgetManager.INVALID_APPWIDGET_ID
                        val resultValue =
                            Intent().putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, appWidgetId)
                        activity?.setResult(RESULT_CANCELED, resultValue)
                        activity?.finish()
                    },
                    text = "Apply",
                )
            }
        }
    }
}

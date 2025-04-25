package net.mullvad.mullvadvpn.screen.widget

import android.app.Activity
import android.app.Activity.RESULT_OK
import android.appwidget.AppWidgetManager
import android.content.Intent
import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.dropUnlessResumed
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.app.widget.MullvadWidgetUpdater
import net.mullvad.mullvadvpn.common.compose.itemWithDivider
import net.mullvad.mullvadvpn.lib.repository.WidgetSettingsState
import net.mullvad.mullvadvpn.lib.ui.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.InfoListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Composable
fun WidgetSettings() {
    val activity = LocalActivity.current
    val appWidgetId =
        activity
            ?.intent
            ?.extras
            ?.getInt(AppWidgetManager.EXTRA_APPWIDGET_ID, AppWidgetManager.INVALID_APPWIDGET_ID)
            ?: AppWidgetManager.INVALID_APPWIDGET_ID
    WidgetSettingsScreen(
        state = runBlocking { MullvadWidgetUpdater.getWidgetConfig(activity!!, appWidgetId) },
        onBackClick = dropUnlessResumed { activity?.finish() },
    )
}

@Composable
fun WidgetSettingsScreen(state: WidgetSettingsState, onBackClick: () -> Unit) {
    var currentState by remember { mutableStateOf(state) }
    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(R.string.widget_settings),
        navigationIcon = { NavigateCloseIconButton(onBackClick) },
    ) { modifier ->
        val lazyListState = rememberLazyListState()
        LazyColumn(modifier = modifier, state = lazyListState) {
            itemWithDivider {
                InfoListItem(title = stringResource(R.string.show_settings_in_widget))
            }
            itemWithDivider {
                WidgetSettingListItem(
                    title =
                        stringResource(
                            R.string.show_setting,
                            stringResource(R.string.local_network_sharing),
                        ),
                    isToggled = currentState.showLan,
                    onCellClicked = { currentState = currentState.copy(showLan = it) },
                )
            }
            itemWithDivider {
                WidgetSettingListItem(
                    title =
                        stringResource(
                            R.string.show_setting,
                            stringResource(R.string.enable_custom_dns),
                        ),
                    isToggled = currentState.showCustomDns,
                    onCellClicked = { currentState = currentState.copy(showCustomDns = it) },
                )
            }
            itemWithDivider {
                WidgetSettingListItem(
                    title = stringResource(R.string.show_setting, stringResource(R.string.daita)),
                    isToggled = currentState.showDaita,
                    onCellClicked = { currentState = currentState.copy(showDaita = it) },
                )
            }
            itemWithDivider {
                WidgetSettingListItem(
                    title =
                        stringResource(
                            R.string.show_setting,
                            stringResource(R.string.split_tunneling),
                        ),
                    isToggled = currentState.showSplitTunneling,
                    onCellClicked = { currentState = currentState.copy(showSplitTunneling = it) },
                )
            }
            itemWithDivider {
                WidgetSettingListItem(
                    title =
                        stringResource(R.string.show_setting, stringResource(R.string.multihop)),
                    isToggled = currentState.showMultihop,
                    onCellClicked = { currentState = currentState.copy(showMultihop = it) },
                )
            }
            itemWithDivider {
                WidgetSettingListItem(
                    title =
                        stringResource(R.string.show_setting, stringResource(R.string.enable_ipv6)),
                    isToggled = currentState.showInTunnelIpv6,
                    onCellClicked = { currentState = currentState.copy(showInTunnelIpv6 = it) },
                )
            }
            itemWithDivider {
                SwitchListItem(
                    title = "Show Quantum-resistant tunnel Setting",
                    isToggled = currentState.showQuantumResistant,
                    onCellClicked = { currentState = currentState.copy(showQuantumResistant = it) },
                )
                WidgetSettingListItem(
                    title =
                        stringResource(
                            R.string.show_setting,
                            stringResource(R.string.quantum_resistant_title),
                        ),
                    isToggled = currentState.showQuantumResistant,
                    onCellClicked = { currentState = currentState.copy(showQuantumResistant = it) },
                )
            }
            item {
                val activity = LocalActivity.current!!
                val scope = rememberCoroutineScope()
                PrimaryButton(
                    modifier =
                        Modifier.padding(
                            horizontal = Dimens.sideMargin,
                            vertical = Dimens.cellVerticalSpacing,
                        ),
                    isEnabled = currentState.anyShowing(),
                    onClick = { onSave(activity, scope, currentState) },
                    text = stringResource(R.string.apply),
                )
            }
        }
    }
}

private fun onSave(activity: Activity, scope: CoroutineScope, state: WidgetSettingsState) {
    val appWidgetId =
        activity.intent
            ?.extras
            ?.getInt(AppWidgetManager.EXTRA_APPWIDGET_ID, AppWidgetManager.INVALID_APPWIDGET_ID)
            ?: AppWidgetManager.INVALID_APPWIDGET_ID

    scope.launch { MullvadWidgetUpdater.updateWidgetWithConfig(activity, appWidgetId, state) }

    val resultValue = Intent().putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, appWidgetId)
    activity.setResult(RESULT_OK, resultValue)
    activity.finish()
}

@Composable
private fun WidgetSettingListItem(
    title: String,
    isToggled: Boolean,
    onCellClicked: (Boolean) -> Unit,
) {
    SwitchListItem(title = title, isToggled = isToggled, onCellClicked = onCellClicked)
}

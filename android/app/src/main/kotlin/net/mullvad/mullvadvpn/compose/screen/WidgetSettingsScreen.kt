package net.mullvad.mullvadvpn.compose.screen

import android.app.Activity
import android.app.Activity.RESULT_OK
import android.appwidget.AppWidgetManager
import android.content.Intent
import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.MaterialTheme
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
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.cell.InformationComposeCell
import net.mullvad.mullvadvpn.compose.cell.NormalSwitchComposeCell
import net.mullvad.mullvadvpn.compose.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.lib.shared.WidgetSettingsState
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.widget.MullvadWidgetUpdater

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
                InformationComposeCell(title = stringResource(R.string.show_settings_in_widget))
            }
            itemWithDivider {
                WidgetSettingCell(
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
                WidgetSettingCell(
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
                WidgetSettingCell(
                    title = stringResource(R.string.show_setting, stringResource(R.string.daita)),
                    isToggled = currentState.showDaita,
                    onCellClicked = { currentState = currentState.copy(showDaita = it) },
                )
            }
            itemWithDivider {
                WidgetSettingCell(
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
                WidgetSettingCell(
                    title =
                        stringResource(R.string.show_setting, stringResource(R.string.multihop)),
                    isToggled = currentState.showMultihop,
                    onCellClicked = { currentState = currentState.copy(showMultihop = it) },
                )
            }
            itemWithDivider {
                WidgetSettingCell(
                    title =
                        stringResource(R.string.show_setting, stringResource(R.string.enable_ipv6)),
                    isToggled = currentState.showInTunnelIpv6,
                    onCellClicked = { currentState = currentState.copy(showInTunnelIpv6 = it) },
                )
            }
            itemWithDivider {
                NormalSwitchComposeCell(
                    title = "Show Quantum-resistant tunnel Setting",
                    isToggled = currentState.showQuantumResistant,
                    onCellClicked = { currentState = currentState.copy(showQuantumResistant = it) },
                    background = MaterialTheme.colorScheme.surfaceContainerLow,
                    startPadding = Dimens.indentedCellStartPadding,
                )
                WidgetSettingCell(
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
private fun WidgetSettingCell(title: String, isToggled: Boolean, onCellClicked: (Boolean) -> Unit) {
    NormalSwitchComposeCell(
        title = title,
        isToggled = isToggled,
        onCellClicked = onCellClicked,
        background = MaterialTheme.colorScheme.surfaceContainerLow,
        startPadding = Dimens.indentedCellStartPadding,
    )
}

package net.mullvad.mullvadvpn.compose.screen

import android.app.Activity.RESULT_CANCELED
import android.appwidget.AppWidgetManager
import android.content.Intent
import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.NavHostGraph
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.cell.InformationComposeCell
import net.mullvad.mullvadvpn.compose.cell.NormalSwitchComposeCell
import net.mullvad.mullvadvpn.compose.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.lib.shared.WidgetSettingsPersister
import net.mullvad.mullvadvpn.lib.shared.WidgetSettingsState
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Destination<WidgetSettingsNavGraph>(start = true)
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
    ) { modifier, lazyListState ->
        LazyColumn(modifier = modifier, state = lazyListState) {
            itemWithDivider { InformationComposeCell(title = "Show settings in widget") }
            itemWithDivider {
                NormalSwitchComposeCell(
                    title = "Show LAN setting",
                    isToggled = state.showLan,
                    onCellClicked = { onToggleShowLan(it) },
                    background = MaterialTheme.colorScheme.surfaceContainerLow,
                    startPadding = Dimens.indentedCellStartPadding,
                )
            }
            itemWithDivider {
                NormalSwitchComposeCell(
                    title = "Show Custom Dns Setting",
                    isToggled = state.showCustomDns,
                    onCellClicked = { onToggleShowCustomDns(it) },
                    background = MaterialTheme.colorScheme.surfaceContainerLow,
                    startPadding = Dimens.indentedCellStartPadding,
                )
            }
            itemWithDivider {
                NormalSwitchComposeCell(
                    title = "Show Daita Setting",
                    isToggled = state.showDaita,
                    onCellClicked = { onToggleShowDaita(it) },
                    background = MaterialTheme.colorScheme.surfaceContainerLow,
                    startPadding = Dimens.indentedCellStartPadding,
                )
            }
            itemWithDivider {
                NormalSwitchComposeCell(
                    title = "Show Split Tunneling Setting",
                    isToggled = state.showSplitTunneling,
                    onCellClicked = { onToggleShowSplitTunneling(it) },
                    background = MaterialTheme.colorScheme.surfaceContainerLow,
                    startPadding = Dimens.indentedCellStartPadding,
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

@NavHostGraph() annotation class WidgetSettingsNavGraph

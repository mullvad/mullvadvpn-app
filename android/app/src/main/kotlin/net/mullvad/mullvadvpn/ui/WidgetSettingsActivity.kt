package net.mullvad.mullvadvpn.ui

import android.appwidget.AppWidgetManager
import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.navigation.NavHostController
import com.ramcosta.composedestinations.DestinationsNavHost
import com.ramcosta.composedestinations.generated.NavGraphs
import com.ramcosta.composedestinations.rememberNavHostEngine
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.viewmodel.MullvadAppViewModel
import org.koin.androidx.compose.koinViewModel

class WidgetSettingsActivity : ComponentActivity() {

    @OptIn(ExperimentalComposeUiApi::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val appWidgetId =
            intent
                ?.extras
                ?.getInt(AppWidgetManager.EXTRA_APPWIDGET_ID, AppWidgetManager.INVALID_APPWIDGET_ID)
                ?: AppWidgetManager.INVALID_APPWIDGET_ID
        val resultValue = Intent().putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, appWidgetId)
        setResult(RESULT_CANCELED, resultValue)

        setContent {
            AppTheme {
                // Widget()

                val engine = rememberNavHostEngine()
                val navHostController: NavHostController = engine.rememberNavController()
                // val navigator: DestinationsNavigator =
                // navHostController.rememberDestinationsNavigator()

                val mullvadAppViewModel = koinViewModel<MullvadAppViewModel>()

                DisposableEffect(Unit) {
                    navHostController.addOnDestinationChangedListener(mullvadAppViewModel)
                    onDispose {
                        navHostController.removeOnDestinationChangedListener(mullvadAppViewModel)
                    }
                }

                DestinationsNavHost(
                    modifier = Modifier.semantics { testTagsAsResourceId = true }.fillMaxSize(),
                    engine = engine,
                    navController = navHostController,
                    navGraph = NavGraphs.widgetSettings,
                )
            }
        }
    }
}

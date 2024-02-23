package net.mullvad.mullvadvpn.compose

import androidx.activity.ComponentActivity
import androidx.compose.runtime.Composable
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.core.view.WindowCompat
import androidx.test.core.app.ActivityScenario
import de.mannodermaus.junit5.compose.ComposeContext
import de.mannodermaus.junit5.compose.createAndroidComposeExtension
import net.mullvad.mullvadvpn.lib.theme.AppTheme

fun ComposeContext.setContentWithTheme(content: @Composable () -> Unit) {
    setContent { AppTheme { content() } }
}

@ExperimentalTestApi
fun createEdgeToEdgeComposeExtension() =
    createAndroidComposeExtension<ComponentActivity>(
        scenarioSupplier = {
            ActivityScenario.launch(ComponentActivity::class.java).onActivity {
                WindowCompat.setDecorFitsSystemWindows(it.window, false)
            } as ActivityScenario<ComponentActivity>
        }
    )

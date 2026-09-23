package net.mullvad.mullvadvpn.screen.test

import android.os.Build
import androidx.activity.ComponentActivity
import androidx.compose.runtime.Composable
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.SemanticsNodeInteractionsProvider
import androidx.compose.ui.test.accessibility.enableAccessibilityChecks
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.tryPerformAccessibilityChecks
import androidx.compose.ui.test.v2.runAndroidComposeUiTest
import androidx.core.view.WindowCompat
import de.mannodermaus.junit5.compose.ComposeContext
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import org.junit.jupiter.api.extension.Extension

fun ComposeContext.setContentWithTheme(content: @Composable () -> Unit) {
    setContent { AppTheme { content() } }
}

@ExperimentalTestApi fun createEdgeToEdgeComposeExtension() = ScreenTestExtension()

@OptIn(ExperimentalTestApi::class)
class ScreenTestExtension : Extension {
    fun use(block: ComposeContext.() -> Unit) {
        runAndroidComposeUiTest(ComponentActivity::class.java) {
            runOnUiThread {
                WindowCompat.setDecorFitsSystemWindows(checkNotNull(activity).window, false)
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
                enableAccessibilityChecks()
            }
            ComposeContextDelegate(this).block()
        }
    }
}

@OptIn(ExperimentalTestApi::class)
private class ComposeContextDelegate(private val test: ComposeUiTest) :
    ComposeContext, SemanticsNodeInteractionsProvider by test {
    override fun setContent(content: @Composable () -> Unit) {
        test.setContent(content)
        test.onRoot().tryPerformAccessibilityChecks()
    }

    override fun waitForIdle() = test.waitForIdle()
}

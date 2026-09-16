package net.mullvad.mullvadvpn.screen.test

import android.annotation.SuppressLint
import androidx.activity.ComponentActivity
import androidx.compose.runtime.Composable
import androidx.compose.ui.test.AndroidComposeUiTestEnvironment
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.SemanticsNodeInteractionsProvider
import androidx.compose.ui.test.accessibility.enableAccessibilityChecks
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.tryPerformAccessibilityChecks
import androidx.core.view.WindowCompat
import androidx.test.core.app.ActivityScenario
import com.google.android.apps.common.testing.accessibility.framework.AccessibilityCheckResult
import com.google.android.apps.common.testing.accessibility.framework.AccessibilityCheckResult.AccessibilityCheckResultType
import com.google.android.apps.common.testing.accessibility.framework.AccessibilityCheckResultUtils.matchesCheck
import com.google.android.apps.common.testing.accessibility.framework.checks.ImageContrastCheck
import com.google.android.apps.common.testing.accessibility.framework.checks.TextContrastCheck
import com.google.android.apps.common.testing.accessibility.framework.checks.TraversalOrderCheck
import com.google.android.apps.common.testing.accessibility.framework.integrations.espresso.AccessibilityValidator
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import org.hamcrest.CoreMatchers.anyOf
import org.junit.jupiter.api.extension.AfterEachCallback
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext

/**
 * The surface a screen test interacts with.
 *
 * Deliberately not [ComposeUiTest]: that type is `@ExperimentalTestApi`, so handing it to tests
 * would require every one of them to opt in. Everything exposed here is stable API, which keeps the
 * experimental surface contained in this module.
 */
interface ScreenTestContext : SemanticsNodeInteractionsProvider {
    fun setContent(content: @Composable () -> Unit)

    fun waitForIdle()
}

fun ScreenTestContext.setContentWithTheme(content: @Composable () -> Unit) {
    setContent { AppTheme { content() } }
}

@ExperimentalTestApi
fun createEdgeToEdgeComposeExtension(): ScreenTestExtension<ComponentActivity> =
    ScreenTestExtension {
        ActivityScenario.launch(ComponentActivity::class.java).onActivity {
            WindowCompat.setDecorFitsSystemWindows(it.window, false)
        }
    }

/**
 * JUnit 5 extension that hosts a Compose test and enables accessibility checks for it.
 *
 * This exists instead of `de.mannodermaus.junit5.compose.createAndroidComposeExtension` because
 * that extension keeps its [ComposeUiTest] private, and [enableAccessibilityChecks] is an extension
 * function on [ComposeUiTest], so the receiver has to be reachable. Everything used here is public
 * API of `androidx.compose.ui:ui-test`.
 *
 * Requires API 34+ on the device, which is what [enableAccessibilityChecks] is annotated with.
 */
@SuppressLint("NewApi")
@OptIn(ExperimentalTestApi::class)
class ScreenTestExtension<A : ComponentActivity>(private val launch: () -> ActivityScenario<A>) :
    BeforeEachCallback, AfterEachCallback {

    private var scenario: ActivityScenario<A>? = null
    private var environment: AndroidComposeUiTestEnvironment<A>? = null

    override fun beforeEach(context: ExtensionContext) {
        environment =
            object : AndroidComposeUiTestEnvironment<A>() {
                override val activity: A?
                    get() {
                        var current: A? = null
                        scenario?.onActivity { current = it }
                        return current
                    }
            }
    }

    fun use(block: ScreenTestContext.() -> Unit) {
        val environment =
            checkNotNull(environment) { "Compose test environment has not been set up" }

        environment.runTest {
            // Launched inside runTest so that content set by the activity itself is picked up.
            scenario = launch()
            enableAccessibilityChecks(accessibilityValidator())
            ScreenTestContextDelegate(this).block()
        }
    }

    override fun afterEach(context: ExtensionContext) {
        scenario?.close()
        scenario = null
        environment = null
    }
}

@OptIn(ExperimentalTestApi::class)
private class ScreenTestContextDelegate(private val test: ComposeUiTest) :
    ScreenTestContext, SemanticsNodeInteractionsProvider by test {

    /**
     * Runs the accessibility checks as soon as content is set.
     *
     * Compose only runs them automatically before actions (`performClick`, `performTextInput`, and
     * so on), so a test that only asserts would never be checked. Doing it here covers the initial
     * rendered state of every screen test, whichever way it sets its content.
     */
    override fun setContent(content: @Composable () -> Unit) {
        test.setContent(content)
        test.onRoot().tryPerformAccessibilityChecks()
    }

    override fun waitForIdle() = test.waitForIdle()
}

/**
 * Fails a test on any accessibility error, currently excluding contrast and traversal order.
 *
 * Those two are excluded so the checks can be turned on without a pre-existing backlog failing
 * every screen test at once. Remove them from the suppression list as that backlog is worked down.
 */
private fun accessibilityValidator(): AccessibilityValidator =
    AccessibilityValidator()
        .setRunChecksFromRootView(true)
        .setThrowExceptionFor(AccessibilityCheckResultType.ERROR)
        .setSuppressingResultMatcher(
            anyOf<AccessibilityCheckResult>(
                matchesCheck(TextContrastCheck::class.java),
                matchesCheck(ImageContrastCheck::class.java),
                matchesCheck(TraversalOrderCheck::class.java),
            )
        )

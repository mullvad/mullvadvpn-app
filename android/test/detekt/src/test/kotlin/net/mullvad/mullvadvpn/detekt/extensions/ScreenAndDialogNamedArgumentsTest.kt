package net.mullvad.mullvadvpn.detekt.extensions

import io.gitlab.arturbosch.detekt.api.Config
import io.gitlab.arturbosch.detekt.test.lint
import net.mullvad.mullvadvpn.detekt.extensions.rules.ScreenAndDialogNamedArguments
import org.junit.jupiter.api.Test

class ScreenAndDialogNamedArgumentsTest {

    private val subject = ScreenAndDialogNamedArguments(Config.empty)

    @Test
    fun `it should find one call that doesn't use only named arguments`() {
        val findings = subject.lint(incorrectCall)
        assert(findings.size == 1)
    }

    @Test
    fun `it should not report an error if all arguments are named`() {
        val findings = subject.lint(correctCall)
        assert(findings.isEmpty())
    }

    @Test
    fun `it should ignore functions that do not end in Screen or Dialog`() {
        val findings = subject.lint(ignoredCall)
        assert(findings.isEmpty())
    }
}

private val incorrectCall: String = """
    @Composable
    fun ExampleComposeScreen(
        arg1: Int,
        arg2: String = "",
    ) {}
    @Composable
    fun Caller() {
        ExampleComposeScreen(2, args2 = "named")
    }
""".trimIndent()

private val correctCall: String = """
    @Composable
    fun ExampleComposeScreen(
        arg1: Int,
        arg2: String = "",
    ) {}
    @Composable
    fun Caller() {
        ExampleComposeScreen(arg1 = 2, args2 = "named")
    }
""".trimIndent()

private val ignoredCall: String = """
    @Composable
    fun ExampleComposable(
        arg1: Int,
        arg2: String = "",
    ) {}
    @Composable
    fun Caller() {
        ExampleComposable(2, args2 = "named")
    }
""".trimIndent()

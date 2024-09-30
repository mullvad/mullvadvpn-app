package net.mullvad.mullvadvpn.test.arch

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.parameters
import com.lemonappdev.konsist.api.provider.KoNameProvider
import com.lemonappdev.konsist.api.verify.assertFalse
import java.util.stream.Stream
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.DynamicTest.dynamicTest
import org.junit.jupiter.api.TestFactory

class NameTest {
    @TestFactory
    fun `ensure no disallowed declaration or parameter names`(): Stream<DynamicTest> =
        DISALLOWED_DECLARATION_OR_PARAMETER_NAMES.stream().map { disallowedName ->
            val testName = "ensure no declarations or parameters include: $disallowedName"
            dynamicTest(testName) {
                Konsist.scopeFromProject()
                    .let { it.declarations() + it.functions().parameters }
                    .filterIsInstance<KoNameProvider>()
                    .assertFalse(testName = testName) { it.hasNameContaining(disallowedName) }
            }
        }

    companion object {
        private val DISALLOWED_DECLARATION_OR_PARAMETER_NAMES =
            listOf("accountNumber", "AccountNumber", "ACCOUNT_NUMBER")
    }
}

package net.mullvad.mullvadvpn.test.arch

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.modifierprovider.withPublicOrDefaultModifier
import com.lemonappdev.konsist.api.verify.assertTrue
import org.junit.jupiter.api.Test

class ManagementServiceTest {

    @Test
    fun `ensure all public functions are returning Either`() {
        managementServiceClass()
            .functions()
            .withPublicOrDefaultModifier()
            .filter { excludedFunctions().contains(it.name).not() }
            .assertTrue { it.returnType?.name?.startsWith(EITHER_CLASS_NAME) == true }
    }

    private fun managementServiceClass() =
        Konsist.scopeFromProject().classes().first { it.name == MANAGEMENT_SERVICE_CLASS_NAME }

    private fun excludedFunctions() = setOf(START, STOP)

    companion object {
        private const val MANAGEMENT_SERVICE_CLASS_NAME = "ManagementService"
        private const val START = "start"
        private const val STOP = "stop"
        private const val EITHER_CLASS_NAME = "Either"
    }
}

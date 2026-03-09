package net.mullvad.mullvadvpn.test.arch

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.withParentNamed
import com.lemonappdev.konsist.api.verify.assertTrue
import org.junit.jupiter.api.Test

class NavigationResultTest {

    @Test
    fun `ensure classes implementing NavigationResult are Serializable`() {
        Konsist.scopeFromProduction()
            .classesAndInterfaces()
            .withParentNamed("NavigationResult", "NavKey")
            .assertTrue(additionalMessage = "Missing @Serializable annotation") {
                it.hasAnnotationWithName("Serializable")
            }
    }
}

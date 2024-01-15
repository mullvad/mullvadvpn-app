package net.mullvad.mullvadvpn.test.arch

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.verify.assertEmpty
import org.junit.jupiter.api.Test

class JUnitTests {

    @Test
    fun `ensure only junit5 annotations are used for functions`() =
        Konsist.scopeFromProject()
            .functions()
            .filter {
                it.annotations.any { annotation ->
                    annotation.fullyQualifiedName.matches(Regex("org.junit((?!jupiter).)*\$"))
                }
            }
            .assertEmpty()

    @Test
    fun `ensure only junit5 annotations are used for classes`() =
        Konsist.scopeFromProject()
            .classes()
            .filter {
                it.annotations.any { annotation ->
                    annotation.fullyQualifiedName.matches(Regex("org.junit((?!jupiter).)*\$"))
                }
            }
            .assertEmpty()
}

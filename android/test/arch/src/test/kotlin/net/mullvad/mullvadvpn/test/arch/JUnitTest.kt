package net.mullvad.mullvadvpn.test.arch

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.withAnnotationOf
import com.lemonappdev.konsist.api.verify.assertEmpty
import com.lemonappdev.konsist.api.verify.assertTrue
import org.junit.jupiter.api.Test

class JUnitTest {

    @Test
    fun `ensure only junit5 annotations are used for functions`() =
        Konsist.scopeFromProject()
            .functions()
            .filter {
                it.annotations.any { annotation ->
                    annotation.fullyQualifiedName?.matches(Regex("org.junit((?!jupiter).)*\$"))
                        ?: false
                }
            }
            .assertEmpty()

    @Test
    fun `ensure only junit5 annotations are used for classes`() =
        Konsist.scopeFromProject()
            .classes()
            .filter {
                it.annotations.any { annotation ->
                    annotation.fullyQualifiedName?.matches(Regex("org.junit((?!jupiter).)*\$"))
                        ?: false
                }
            }
            .assertEmpty()

    @Test
    fun `ensure all non android tests are written with spaces`() =
        allNonAndroidTests().assertTrue { it.name.contains(' ') }

    @Test
    fun `ensure all non android tests does start with lower case letter`() =
        allNonAndroidTests().assertTrue { it.name.first().isLowerCase() }

    @Test
    fun `ensure all non android tests have 'ensure' or 'should' in function name`() =
        allNonAndroidTests().assertTrue { it.name.containsEnsureOrShould() }

    private fun String.containsEnsureOrShould(): Boolean {
        return contains("ensure") || contains("should") || contains("then")
    }

    private fun allNonAndroidTests() =
        Konsist.scopeFromTest().functions().withAnnotationOf(Test::class).filter {
            it.sourceSetName != "androidTest"
        }
}

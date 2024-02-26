package net.mullvad.mullvadvpn.test.arch

import com.lemonappdev.konsist.api.Konsist
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
        return contains("ensure") || contains("should")
    }

    private fun allNonAndroidTests() =
        Konsist.scopeFromTest()
            .functions()
            // withAnnotationOf is broken in latest Consist version, so we filter manually
            // https://github.com/LemonAppDev/konsist/discussions/738
            .filter { it.annotations.any { it.text == "@Test" } }
            .filter { it.sourceSetName != "androidTest" }
            .filter { function ->
                ignoredTestPackages.none { function.packagee!!.fullyQualifiedName.startsWith(it) }
            }

    companion object {
        // The following packages are not following the naming convention since they are android
        // test that does not support spaces in function names.
        private val ignoredTestPackages =
            listOf("net.mullvad.mullvadvpn.test.e2e", "net.mullvad.mullvadvpn.test.mockapi")
    }
}

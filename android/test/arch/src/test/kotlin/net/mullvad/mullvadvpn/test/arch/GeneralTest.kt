package net.mullvad.mullvadvpn.test.arch

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.properties
import com.lemonappdev.konsist.api.verify.assertFalse
import com.lemonappdev.konsist.api.verify.assertNot
import com.lemonappdev.konsist.api.verify.assertTrue
import org.junit.jupiter.api.Test

class GeneralTest {
    @Test
    fun `ensure package name must match file path`() =
        Konsist.scopeFromProject().packages.assertTrue { it.hasMatchingPath }

    @Test
    fun `ensure no field should have 'm' prefix`() =
        Konsist.scopeFromProject().classes().properties().assertNot {
            val secondCharacterIsUppercase = it.name.getOrNull(1)?.isUpperCase() ?: false
            it.name.startsWith('m') && secondCharacterIsUppercase
        }

    @Test
    fun `ensure no empty files allowed`() =
        Konsist.scopeFromProject().files.assertFalse { it.text.isEmpty() }
}

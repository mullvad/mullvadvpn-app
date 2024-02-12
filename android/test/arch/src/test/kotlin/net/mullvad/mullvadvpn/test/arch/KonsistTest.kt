package net.mullvad.mullvadvpn.test.arch

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.ext.list.withAnnotationOf
import com.lemonappdev.konsist.api.verify.assertTrue
import org.junit.jupiter.api.Test

class KonsistTest {
    @Test
    fun `ensure konsist tests have 'ensure ' prefix`() =
        Konsist.scopeFromModule("test/arch").functions().withAnnotationOf(Test::class).assertTrue {
            it.hasNameStartingWith("ensure ")
        }
}

package net.mullvad.mullvadvpn.test.arch

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.verify.assertFalse
import org.junit.jupiter.api.Test

class ForbiddenApiTest {
    // Avoid usage of ActivityResultContracts.GetContent() since
    // it doesn't work with some file managers. See: DROID-2692
    @Test
    fun `ensure ActivityResultContracts GetContent is not used`() =
        Konsist.scopeFromProduction().files.assertFalse {
            it.text.contains("ActivityResultContracts.GetContent()")
        }
}

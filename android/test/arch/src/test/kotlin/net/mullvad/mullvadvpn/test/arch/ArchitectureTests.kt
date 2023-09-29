package net.mullvad.mullvadvpn.test.arch

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.architecture.KoArchitectureCreator.assertArchitecture
import com.lemonappdev.konsist.api.architecture.Layer
import org.junit.Test

class ArchitectureTests {

    @Test
    fun `domain layer depends on nothing`() {
        Konsist.scopeFromProduction().assertArchitecture {
            val domain = Layer("Domain", "net.mullvad.mullvadvpn.model..")

            domain.dependsOnNothing()
        }
    }
}

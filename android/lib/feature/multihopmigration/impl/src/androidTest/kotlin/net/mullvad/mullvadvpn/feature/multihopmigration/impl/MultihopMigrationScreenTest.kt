package net.mullvad.mullvadvpn.feature.filter.impl

import androidx.compose.ui.test.ExperimentalTestApi
import de.mannodermaus.junit5.compose.ComposeContext
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import net.mullvad.mullvadvpn.screen.test.setContentWithTheme
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@OptIn(ExperimentalTestApi::class)
class MultihopMigrationScreenTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    private fun ComposeContext.initScreen(
    ) {
        setContentWithTheme {
        }
    }

    @Test
    fun testDefaultState() = composeExtension.use {
    }

}

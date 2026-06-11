package net.mullvad.mullvadvpn.lib.common.compose

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithContentDescription
import net.mullvad.mullvadvpn.lib.ui.icon.DeleteHistory
import net.mullvad.mullvadvpn.lib.ui.icon.MultihopWhenNeeded
import net.mullvad.mullvadvpn.screen.test.createEdgeToEdgeComposeExtension
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.RegisterExtension

@ExperimentalTestApi
@OptIn(ExperimentalMaterial3Api::class)
class IconStringTest {
    @JvmField @RegisterExtension val composeExtension = createEdgeToEdgeComposeExtension()

    @Test
    fun testMultipleIconsInString() = composeExtension.use {
        // Arrange
        setContent {
            val iconString =
                stringResourceWithIcons(
                    id = R.string.multiple_icons,
                    DescribedIcon(icon = MultihopWhenNeeded, contentDescription = "icon1"),
                    DescribedIcon(icon = DeleteHistory, contentDescription = "icon2"),
                )
            Text(text = iconString.text, inlineContent = iconString.inlineContent)
        }

        // Assert
        onNodeWithContentDescription("icon1").assertExists()
        onNodeWithContentDescription("icon2").assertExists()
    }

    @Test
    fun testIconAtEndOfString() = composeExtension.use {
        // Arrange
        setContent {
            val iconString =
                stringResourceWithIcons(
                    id = R.string.icon_at_end,
                    DescribedIcon(icon = MultihopWhenNeeded, contentDescription = "icon1"),
                )
            Text(text = iconString.text, inlineContent = iconString.inlineContent)
        }

        // Assert
        onNodeWithContentDescription("icon1").assertExists()
    }
}

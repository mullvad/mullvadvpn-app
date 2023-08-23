package net.mullvad.mullvadvpn.lib.theme.typeface

import androidx.compose.material3.Typography
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit

// Add text styles not in the material theme here
val Typography.listItemText: TextStyle
    @Composable
    get() {
        return TextStyle(
            fontWeight = FontWeight.Normal,
            letterSpacing = TextUnit.Unspecified,
            fontSize = TypeScale.TextMediumPlus
        )
    }

val Typography.listItemSubText: TextStyle
    @Composable
    get() {
        return TextStyle(
            fontWeight = FontWeight.SemiBold,
            letterSpacing = TextUnit.Unspecified,
            fontSize = TypeScale.TextSmall
        )
    }

val Typography.connectionStatus: TextStyle
    @Composable
    get() {
        return TextStyle(
            fontWeight = FontWeight.Bold,
            letterSpacing = TextUnit.Unspecified,
            fontSize = TypeScale.TextMedium
        )
    }

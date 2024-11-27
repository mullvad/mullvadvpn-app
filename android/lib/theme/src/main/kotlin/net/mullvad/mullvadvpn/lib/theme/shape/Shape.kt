package net.mullvad.mullvadvpn.lib.theme.shape

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Shapes
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.unit.dp

val Shapes.chipShape: Shape
    @Composable
    get() {
        return RoundedCornerShape(8.dp)
    }

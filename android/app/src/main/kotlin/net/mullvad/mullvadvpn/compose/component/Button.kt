package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonColors
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R

@Composable
fun ActionButton(
    text: String,
    onClick: () -> Unit,
    colors: ButtonColors,
    modifier: Modifier = Modifier,
    isEnabled: Boolean = true
) {
    Button(
        onClick = onClick,
        enabled = isEnabled,
        // Required along with defaultMinSize to control size and padding.
        contentPadding = PaddingValues(0.dp),
        modifier =
            modifier
                .height(dimensionResource(id = R.dimen.button_height))
                .defaultMinSize(
                    minWidth = 0.dp,
                    minHeight = dimensionResource(id = R.dimen.button_height)
                )
                .fillMaxWidth(),
        colors = colors,
        shape = MaterialTheme.shapes.small
    ) {
        Text(
            text = text,
            textAlign = TextAlign.Center,
            fontSize = 18.sp,
            fontWeight = FontWeight.Bold
        )
    }
}

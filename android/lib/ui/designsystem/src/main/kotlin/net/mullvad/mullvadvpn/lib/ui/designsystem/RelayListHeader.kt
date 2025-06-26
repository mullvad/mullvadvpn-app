package net.mullvad.mullvadvpn.lib.ui.designsystem

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Composable
fun RelayListHeader(
    content: @Composable () -> Unit,
    modifier: Modifier = Modifier,
    actions: @Composable (RowScope.() -> Unit)? = null,
    colors: RelayListItemColors = RelayListItemDefaults.colors(),
) {
    ProvideContentColorTextStyle(
        MaterialTheme.colorScheme.onBackground,
        MaterialTheme.typography.bodyLarge,
    ) {
        Row(
            modifier =
                Modifier.defaultMinSize(minHeight = 48.dp).height(IntrinsicSize.Min).then(modifier),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            content()
            HorizontalDivider(
                Modifier.weight(1f, true)
                    .padding(start = 8.dp),
                color = MaterialTheme.colorScheme.onBackground.copy(alpha = 0.2f)
            )
            actions?.invoke(this)
        }
    }
}

@Preview(backgroundColor = 0xFF192E45, showBackground = true)
@Composable
fun PreviewRelayListHeader() {

    AppTheme {
        Column {
            RelayListHeader(content = { Text("Header") })
            RelayListHeader(
                content = { Text("Header") },
                actions = {
                    IconButton(onClick = {}) {
                        Icon(imageVector = Icons.Default.Edit, contentDescription = null)
                    }
                },
            )
        }
    }
}

package net.mullvad.mullvadvpn.lib.ui.designsystem.preview

import android.annotation.SuppressLint
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Composable
internal fun PreviewColumn(
    @SuppressLint("ModifierParameter") modifier: Modifier = Modifier.padding(Dimens.mediumPadding),
    spacing: Dp = Dimens.mediumSpacer,
    verticalAlignment: Alignment.Vertical = Alignment.Top,
    horizontalAlignment: Alignment.Horizontal = Alignment.Start,
    content: @Composable ColumnScope.() -> Unit,
) {
    AppTheme {
        Column(
            modifier = modifier,
            verticalArrangement = Arrangement.spacedBy(spacing, verticalAlignment),
            horizontalAlignment = horizontalAlignment,
            content = content,
        )
    }
}

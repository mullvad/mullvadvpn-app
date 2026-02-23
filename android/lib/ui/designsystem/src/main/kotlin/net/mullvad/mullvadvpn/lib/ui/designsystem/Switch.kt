package net.mullvad.mullvadvpn.lib.ui.designsystem

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchColors
import androidx.compose.material3.SwitchDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.ui.designsystem.preview.PreviewColumn
import net.mullvad.mullvadvpn.lib.ui.tag.SWITCH_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.ui.theme.color.positive

@Preview(backgroundColor = 0xFF192E45, showBackground = true)
@Composable
private fun PreviewMullvadSwitch() {
    PreviewColumn() {
        MullvadSwitch(checked = true, onCheckedChange = null)
        MullvadSwitch(checked = false, onCheckedChange = null)
        MullvadSwitch(checked = true, onCheckedChange = null, enabled = false)
        MullvadSwitch(checked = false, onCheckedChange = null, enabled = false)
    }
}

@Composable
fun MullvadSwitch(
    checked: Boolean,
    onCheckedChange: ((Boolean) -> Unit)?,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    colors: SwitchColors = mullvadSwitchColors(),
    interactionSource: MutableInteractionSource = remember { MutableInteractionSource() },
    content: @Composable (() -> Unit)? = {
        // This is needed to ensure the thumb always is big in off mode
        Spacer(modifier = Modifier.size(Dimens.switchIconSize))
    },
) {
    Switch(
        checked = checked,
        onCheckedChange = onCheckedChange,
        modifier = modifier.testTag(SWITCH_TEST_TAG),
        thumbContent = content,
        enabled = enabled,
        colors = colors,
        interactionSource = interactionSource,
    )
}

@Composable
fun mullvadSwitchColors(): SwitchColors =
    SwitchDefaults.colors(
        checkedThumbColor = MaterialTheme.colorScheme.positive,
        checkedTrackColor = Color.Transparent,
        checkedBorderColor = MaterialTheme.colorScheme.onPrimary,
        uncheckedThumbColor = MaterialTheme.colorScheme.error,
        uncheckedTrackColor = Color.Transparent,
        uncheckedBorderColor = MaterialTheme.colorScheme.onPrimary,
        disabledCheckedThumbColor = MaterialTheme.colorScheme.positive.copy(alpha = AlphaDisabled),
        disabledCheckedTrackColor = Color.Transparent,
        disabledCheckedBorderColor =
            MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled),
        disabledUncheckedThumbColor = MaterialTheme.colorScheme.error.copy(alpha = AlphaDisabled),
        disabledUncheckedTrackColor = Color.Transparent,
        disabledUncheckedBorderColor =
            MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled),
    )

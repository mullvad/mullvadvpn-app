package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchColors
import androidx.compose.material3.SwitchDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled

@Preview
@Composable
private fun PreviewMullvadSwitch() {
    AppTheme {
        Surface(color = MaterialTheme.colorScheme.background) {
            Column(
                verticalArrangement = Arrangement.spacedBy(Dimens.mediumPadding),
                modifier = Modifier.padding(Dimens.sideMargin)
            ) {
                MullvadSwitch(checked = true, onCheckedChange = null)
                MullvadSwitch(checked = false, onCheckedChange = null)
                MullvadSwitch(checked = true, enabled = false, onCheckedChange = null)
                MullvadSwitch(checked = false, enabled = false, onCheckedChange = null)
            }
        }
    }
}

@Composable
fun MullvadSwitch(
    checked: Boolean,
    onCheckedChange: ((Boolean) -> Unit)?,
    modifier: Modifier = Modifier,
    thumbContent: (@Composable () -> Unit)? = {
        // This is needed to ensure the thumb always is big in off mode
        Spacer(modifier = Modifier.size(24.dp))
    },
    enabled: Boolean = true,
    colors: SwitchColors = mullvadSwitchColors(),
    interactionSource: MutableInteractionSource = remember { MutableInteractionSource() },
) {
    Switch(
        checked = checked,
        onCheckedChange = onCheckedChange,
        modifier = modifier,
        thumbContent = thumbContent,
        enabled = enabled,
        colors = colors,
        interactionSource
    )
}

@Composable
fun mullvadSwitchColors(): SwitchColors =
    SwitchDefaults.colors(
        checkedThumbColor = MaterialTheme.colorScheme.surface, // TODO Change
        checkedTrackColor = MaterialTheme.colorScheme.primary,
        checkedBorderColor = MaterialTheme.colorScheme.onPrimary,
        //    checkedIconColor= SwitchTokens.SelectedIconColor.toColor(),
        uncheckedThumbColor = MaterialTheme.colorScheme.error,
        uncheckedTrackColor = MaterialTheme.colorScheme.primary,
        uncheckedBorderColor = MaterialTheme.colorScheme.onPrimary,
        //    uncheckedIconColor= SwitchTokens.UnselectedIconColor.toColor(),
        disabledCheckedThumbColor =
            MaterialTheme.colorScheme.surface
                .copy(alpha = AlphaDisabled)
                .compositeOver(MaterialTheme.colorScheme.primary),
        disabledCheckedTrackColor = MaterialTheme.colorScheme.primary,
        disabledCheckedBorderColor =
            MaterialTheme.colorScheme.onPrimary
                .copy(alpha = AlphaDisabled)
                .compositeOver(MaterialTheme.colorScheme.primary),
        disabledUncheckedThumbColor =
            MaterialTheme.colorScheme.error
                .copy(alpha = AlphaDisabled)
                .compositeOver(MaterialTheme.colorScheme.primary),
        disabledUncheckedTrackColor = MaterialTheme.colorScheme.primary,
        disabledUncheckedBorderColor =
            MaterialTheme.colorScheme.onPrimary
                .copy(alpha = AlphaDisabled)
                .compositeOver(MaterialTheme.colorScheme.primary),
    )

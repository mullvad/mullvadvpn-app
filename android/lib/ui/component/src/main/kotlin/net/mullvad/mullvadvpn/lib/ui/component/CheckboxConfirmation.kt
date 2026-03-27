package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.lib.ui.designsystem.Checkbox
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListTokens
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Composable
fun CheckboxConfirmation(
    text: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
    checkedExtraContent: @Composable (() -> Unit)? = null,
) {
    Column(
        modifier =
            modifier
                .animateContentSize()
                .border(
                    width = Dp.Hairline,
                    color = MaterialTheme.colorScheme.primary,
                    shape = MaterialTheme.shapes.medium,
                )
                .clip(shape = MaterialTheme.shapes.medium)
                .padding(
                    bottom =
                        if (checkedExtraContent != null && checked) Dimens.smallPadding
                        else Dimens.tinyPadding,
                    end = Dimens.tinyPadding,
                    start = Dimens.tinyPadding,
                    top = Dimens.tinyPadding,
                )
    ) {
        Row(
            modifier =
                Modifier.defaultMinSize(minHeight = ListTokens.listItemMinHeight)
                    .fillMaxWidth()
                    .clickable(onClick = { onCheckedChange(!checked) }),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Checkbox(
                modifier = Modifier.padding(end = Dimens.smallPadding),
                checked = checked,
                onCheckedChange = onCheckedChange,
            )
            Text(style = MaterialTheme.typography.bodyMedium, text = text)
        }
        if (checked) {
            checkedExtraContent?.invoke()
        }
    }
}

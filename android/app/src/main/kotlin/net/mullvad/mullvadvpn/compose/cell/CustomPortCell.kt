package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.Alpha20
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInvisible
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.theme.color.selected

@Preview
@Composable
private fun PreviewCustomPortCell() {
    AppTheme {
        SpacedColumn(Modifier.background(MaterialTheme.colorScheme.background)) {
            CustomPortCell(title = "Title", isSelected = true, port = 444)
            CustomPortCell(title = "Title", isSelected = false, port = null)
        }
    }
}

@Composable
fun CustomPortCell(
    title: String,
    isSelected: Boolean,
    port: Int?,
    mainTestTag: String = "",
    numberTestTag: String = "",
    onMainCellClicked: () -> Unit = {},
    onPortCellClicked: () -> Unit = {}
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.Start,
        modifier = Modifier.height(Dimens.cellHeight).fillMaxWidth()
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Start,
            modifier =
                Modifier.clickable { onMainCellClicked() }
                    .height(Dimens.cellHeight)
                    .weight(1f)
                    .background(
                        if (isSelected) {
                            MaterialTheme.colorScheme.selected
                        } else {
                            MaterialTheme.colorScheme.primaryContainer
                                .copy(alpha = Alpha20)
                                .compositeOver(MaterialTheme.colorScheme.background)
                        }
                    )
                    .padding(start = Dimens.cellStartPadding)
                    .testTag(mainTestTag)
        ) {
            Icon(
                painter = painterResource(id = R.drawable.icon_tick),
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onPrimary,
                modifier =
                    Modifier.padding(end = Dimens.selectableCellTextMargin)
                        .alpha(if (isSelected) AlphaVisible else AlphaInvisible)
            )
            BaseCellTitle(
                title = title,
                style = MaterialTheme.typography.labelLarge,
                textAlign = TextAlign.Start
            )
        }
        Spacer(modifier = Modifier.width(Dimens.verticalSpacer))
        Box(
            modifier =
                Modifier.clickable { onPortCellClicked() }
                    .height(Dimens.cellHeight)
                    .wrapContentWidth()
                    .defaultMinSize(minWidth = Dimens.customPortBoxMinWidth)
                    .background(MaterialTheme.colorScheme.primary)
                    .testTag(numberTestTag)
        ) {
            Text(
                text = port?.toString() ?: stringResource(id = R.string.port),
                color = MaterialTheme.colorScheme.onPrimary,
                modifier = Modifier.align(Alignment.Center)
            )
        }
    }
}

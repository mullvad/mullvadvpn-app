package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.Alpha20

@Preview
@Composable
private fun PreviewMullvadProgressIndicator() {
    AppTheme {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            MullvadCircularProgressIndicatorLarge()
            MullvadCircularProgressIndicatorMedium()
            MullvadCircularProgressIndicatorSmall()
        }
    }
}

@Composable
fun MullvadCircularProgressIndicatorLarge(
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.onBackground,
    trackColor: Color = color.copy(alpha = Alpha20),
) {
    CircularProgressIndicator(
        modifier.size(Dimens.circularProgressBarLargeSize),
        color,
        Dimens.circularProgressBarLargeStrokeWidth,
        trackColor,
        StrokeCap.Round
    )
}

@Composable
fun MullvadCircularProgressIndicatorMedium(
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.onBackground,
    trackColor: Color = color.copy(alpha = Alpha20),
) {
    CircularProgressIndicator(
        modifier.size(Dimens.circularProgressBarMediumSize),
        color,
        Dimens.circularProgressBarMediumStrokeWidth,
        trackColor,
        StrokeCap.Round
    )
}

@Composable
fun MullvadCircularProgressIndicatorSmall(
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.onBackground,
    trackColor: Color = color.copy(alpha = Alpha20),
) {
    CircularProgressIndicator(
        modifier.size(Dimens.circularProgressBarSmallSize),
        color,
        Dimens.circularProgressBarSmallStrokeWidth,
        trackColor,
        StrokeCap.Round
    )
}

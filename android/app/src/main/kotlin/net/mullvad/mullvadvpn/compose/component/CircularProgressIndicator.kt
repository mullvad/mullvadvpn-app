package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.tag.CIRCULAR_PROGRESS_INDICATOR_TEST_TAG

@Preview
@Composable
private fun PreviewMullvadProgressIndicator() {
    AppTheme {
        Column(
            modifier = Modifier.padding(16.dp).background(MaterialTheme.colorScheme.surface),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            MullvadCircularProgressIndicatorLarge()
            MullvadCircularProgressIndicatorMedium()
            MullvadCircularProgressIndicatorSmall()
        }
    }
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun MullvadCircularProgressIndicatorLarge(
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.onSurface,
    // trackColor: Color = color.copy(alpha = Alpha20),
) {
    LoadingIndicator(
        modifier =
            modifier
                //.padding(Dimens.tinyPadding)
                //.size(Dimens.circularProgressBarLargeSize)
                .testTag(CIRCULAR_PROGRESS_INDICATOR_TEST_TAG),
        color = color,
    )
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun MullvadCircularProgressIndicatorMedium(
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.onSurface,
    // trackColor: Color = color.copy(alpha = Alpha20),
) {
    LoadingIndicator(
        modifier =
            modifier
                .size(Dimens.circularProgressBarMediumSize)
                .testTag(CIRCULAR_PROGRESS_INDICATOR_TEST_TAG),
        color = color,
    )
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun MullvadCircularProgressIndicatorSmall(
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.onSurface,
    // trackColor: Color = color.copy(alpha = Alpha20),
) {
    LoadingIndicator(
        modifier =
            modifier
                .size(Dimens.circularProgressBarSmallSize)
                .testTag(CIRCULAR_PROGRESS_INDICATOR_TEST_TAG),
        color = color,
    )
}

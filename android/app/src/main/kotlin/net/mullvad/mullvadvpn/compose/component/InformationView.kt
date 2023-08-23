package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.typeface.TypeScale.TextMedium
import net.mullvad.mullvadvpn.lib.theme.typeface.TypeScale.TextSmall

@Preview
@Composable
private fun PreviewInformationView() {
    InformationView(content = "test content")
}

@Preview
@Composable
private fun PreviewEmptyInformationView() {
    InformationView(content = "", whenMissing = MissingPolicy.SHOW_SPINNER)
}

@Composable
fun InformationView(
    content: String,
    modifier: Modifier = Modifier,
    whenMissing: MissingPolicy = MissingPolicy.SHOW_VIEW,
    maxLines: Int = 1
) {
    return if (content.isNotEmpty()) {
        AutoResizeText(
            style = MaterialTheme.typography.titleSmall,
            text = content,
            minTextSize = TextSmall,
            maxTextSize = TextMedium,
            maxLines = maxLines,
            modifier =
                modifier.padding(
                    start = Dimens.sideMargin,
                    top = Dimens.smallPadding,
                    bottom = Dimens.smallPadding
                )
        )
    } else {
        when (whenMissing) {
            MissingPolicy.SHOW_VIEW -> {
                AutoResizeText(
                    style = MaterialTheme.typography.titleMedium,
                    text = content,
                    minTextSize = TextSmall,
                    maxTextSize = TextMedium,
                    maxLines = maxLines,
                    modifier =
                        modifier.padding(
                            start = Dimens.sideMargin,
                            top = Dimens.smallPadding,
                            bottom = Dimens.smallPadding
                        )
                )
            }
            MissingPolicy.HIDE_VIEW -> {}
            MissingPolicy.SHOW_SPINNER -> {
                Row(verticalAlignment = Alignment.CenterVertically, modifier = modifier) {
                    CircularProgressIndicator(
                        modifier =
                            Modifier.padding(
                                    start = Dimens.sideMargin,
                                    top = Dimens.smallPadding,
                                    bottom = Dimens.smallPadding
                                )
                                .height(Dimens.loadingSpinnerSizeMedium)
                                .width(Dimens.loadingSpinnerSizeMedium),
                        color = MaterialTheme.colorScheme.onSecondary
                    )
                    Spacer(modifier = Modifier.weight(1f))
                }
            }
        }
    }
}

enum class MissingPolicy {
    SHOW_VIEW,
    HIDE_VIEW,
    SHOW_SPINNER
}

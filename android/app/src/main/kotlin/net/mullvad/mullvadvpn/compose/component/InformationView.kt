package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.compose.theme.Dimens

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
fun InformationView(content: String, whenMissing: MissingPolicy = MissingPolicy.SHOW_VIEW) {
    return if (content.isNotEmpty()) {
        Text(
            style = MaterialTheme.typography.titleSmall,
            text = content,
            modifier =
                Modifier.padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    top = Dimens.smallPadding,
                    bottom = Dimens.mediumPadding
                )
        )
    } else {
        when (whenMissing) {
            MissingPolicy.SHOW_VIEW -> {
                Text(
                    style = MaterialTheme.typography.titleMedium,
                    text = content,
                    modifier =
                        Modifier.padding(
                            start = Dimens.sideMargin,
                            end = Dimens.sideMargin,
                            top = Dimens.smallPadding,
                            bottom = Dimens.mediumPadding
                        )
                )
            }
            MissingPolicy.HIDE_VIEW -> {}
            MissingPolicy.SHOW_SPINNER -> {
                CircularProgressIndicator(
                    modifier =
                        Modifier.padding(
                                start = Dimens.sideMargin,
                                end = Dimens.sideMargin,
                                top = Dimens.smallPadding,
                                bottom = Dimens.mediumPadding
                            )
                            .height(Dimens.loadingSpinnerSizeMedium)
                            .width(Dimens.loadingSpinnerSizeMedium),
                    color = MaterialTheme.colorScheme.onSecondary
                )
            }
        }
    }
}

enum class MissingPolicy {
    SHOW_VIEW,
    HIDE_VIEW,
    SHOW_SPINNER
}

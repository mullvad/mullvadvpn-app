package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.core.text.HtmlCompat
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.relaylist.MIN_SEARCH_LENGTH

@Composable
fun LocationsEmptyText(searchTerm: String) {
    if (searchTerm.length >= MIN_SEARCH_LENGTH) {
        val firstRow =
            HtmlCompat.fromHtml(
                    textResource(
                        id = R.string.select_location_empty_text_first_row,
                        searchTerm,
                    ),
                    HtmlCompat.FROM_HTML_MODE_COMPACT,
                )
                .toAnnotatedString(boldFontWeight = FontWeight.ExtraBold)
        val secondRow = textResource(id = R.string.select_location_empty_text_second_row)
        Column(
            modifier = Modifier.padding(horizontal = Dimens.selectLocationTitlePadding),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                text = firstRow,
                style = MaterialTheme.typography.labelMedium,
                textAlign = TextAlign.Center,
                color = MaterialTheme.colorScheme.onSecondary,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = secondRow,
                style = MaterialTheme.typography.labelMedium,
                textAlign = TextAlign.Center,
                color = MaterialTheme.colorScheme.onSecondary,
            )
        }
    } else {
        Text(
            text = stringResource(R.string.no_locations_found),
            modifier = Modifier.padding(Dimens.screenVerticalMargin),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSecondary
        )
    }
}

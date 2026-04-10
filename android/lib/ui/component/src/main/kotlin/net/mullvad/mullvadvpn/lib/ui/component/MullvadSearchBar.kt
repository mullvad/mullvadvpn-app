package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SearchBarDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.lib.ui.component.textfield.mullvadDarkTextFieldColors
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MullvadSearchBar(
    searchTerm: String,
    enabled: Boolean,
    onSearchInputChanged: (String) -> Unit,
    hideKeyboard: () -> Unit,
    onGoBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    SearchBarDefaults.InputField(
        modifier = modifier.height(Dimens.searchFieldHeightExpanded).fillMaxWidth(),
        query = searchTerm,
        enabled = enabled,
        onQueryChange = onSearchInputChanged,
        onSearch = { hideKeyboard() },
        expanded = true,
        onExpandedChange = {},
        leadingIcon = {
            IconButton(onClick = onGoBack) {
                Icon(
                    imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                    contentDescription = stringResource(R.string.back),
                )
            }
        },
        trailingIcon = {
            if (searchTerm.isNotEmpty()) {
                IconButton(onClick = { onSearchInputChanged("") }) {
                    Icon(
                        imageVector = Icons.Rounded.Close,
                        contentDescription = stringResource(R.string.clear_input),
                    )
                }
            }
        },
        placeholder = { Text(text = stringResource(id = R.string.search_placeholder)) },
        colors =
            mullvadDarkTextFieldColors()
                .copy(
                    focusedContainerColor = MaterialTheme.colorScheme.surface,
                    unfocusedContainerColor = MaterialTheme.colorScheme.surface,
                    errorContainerColor = MaterialTheme.colorScheme.surface,
                    disabledContainerColor = MaterialTheme.colorScheme.surface,
                    disabledLeadingIconColor = MaterialTheme.colorScheme.onSurface,
                ),
    )
}

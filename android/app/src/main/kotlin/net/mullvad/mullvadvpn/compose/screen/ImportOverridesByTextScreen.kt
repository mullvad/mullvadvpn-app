package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.result.ResultBackNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.MullvadSmallTopBar
import net.mullvad.mullvadvpn.compose.textfield.mullvadWhiteTextFieldColors
import net.mullvad.mullvadvpn.compose.transitions.DefaultTransition

@Preview
@Composable
private fun PreviewImportOverridesByText() {
    ImportOverridesByTextScreen({}, {})
}

@Destination(style = DefaultTransition::class)
@Composable
fun ImportOverridesByText(
    resultNavigator: ResultBackNavigator<String>,
) {
    ImportOverridesByTextScreen(
        onNavigateBack = resultNavigator::navigateBack,
        onImportClicked = { resultNavigator.navigateBack(result = it) }
    )
}

@Composable
fun ImportOverridesByTextScreen(
    onNavigateBack: () -> Unit,
    onImportClicked: (String) -> Unit,
) {
    var text by remember { mutableStateOf("") }

    Scaffold(
        topBar = {
            MullvadSmallTopBar(
                title = stringResource(R.string.import_overrides_text_title),
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(imageVector = Icons.Default.Close, contentDescription = null)
                    }
                },
                actions = {
                    TextButton(
                        enabled = text.isNotEmpty(),
                        colors =
                            ButtonDefaults.textButtonColors()
                                .copy(contentColor = MaterialTheme.colorScheme.onPrimary),
                        onClick = { onImportClicked(text) }
                    ) {
                        Text(
                            text = stringResource(R.string.import_overrides_import),
                        )
                    }
                }
            )
        },
    ) {
        Column(modifier = Modifier.padding(it)) {
            TextField(
                modifier = Modifier.fillMaxSize(),
                value = text,
                onValueChange = { text = it },
                placeholder = {
                    Text(text = stringResource(R.string.import_override_textfield_placeholder))
                },
                colors = mullvadWhiteTextFieldColors()
            )
        }
    }
}

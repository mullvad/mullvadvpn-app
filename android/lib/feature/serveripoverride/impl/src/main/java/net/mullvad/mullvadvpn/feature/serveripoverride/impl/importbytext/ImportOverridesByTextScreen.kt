package net.mullvad.mullvadvpn.feature.serveripoverride.impl.importbytext

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextDirection
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ImportOverrideByTextNavResult
import net.mullvad.mullvadvpn.lib.ui.component.MullvadSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.textfield.mullvadWhiteTextFieldColors
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadSnackbar
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_IMPORT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDES_TEXT_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewImportOverridesByText() {
    AppTheme {
        ImportOverridesByTextScreen(
            snackbarHostState = SnackbarHostState(),
            onNavigateBack = {},
            onImportClicked = {},
        )
    }
}

@Composable
fun ImportOverridesByText(navigator: Navigator) {
    val snackbarHostState = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    val resources = LocalResources.current
    ImportOverridesByTextScreen(
        snackbarHostState = snackbarHostState,
        onNavigateBack = dropUnlessResumed { navigator.goBack() },
        onImportClicked = { text ->
            if (text.isNotBlank()) {
                navigator.goBack(result = ImportOverrideByTextNavResult(text))
            } else {
                scope.launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = resources.getString(R.string.import_overrides_empty_text_error)
                    )
                }
            }
        },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ImportOverridesByTextScreen(
    snackbarHostState: SnackbarHostState,
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
                        Icon(
                            imageVector = Icons.Rounded.Close,
                            contentDescription = stringResource(id = R.string.close),
                        )
                    }
                },
                actions = {
                    TextButton(
                        modifier =
                            Modifier.testTag(
                                SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_IMPORT_BUTTON_TEST_TAG
                            ),
                        colors =
                            ButtonDefaults.textButtonColors()
                                .copy(contentColor = MaterialTheme.colorScheme.onPrimary),
                        onClick = dropUnlessResumed { onImportClicked(text) },
                    ) {
                        Text(text = stringResource(R.string.import_overrides_import))
                    }
                },
            )
        },
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) },
            )
        },
    ) {
        Column(modifier = Modifier.padding(it)) {
            TextField(
                modifier = Modifier.fillMaxSize().testTag(SERVER_IP_OVERRIDES_TEXT_INPUT_TEST_TAG),
                value = text,
                onValueChange = { text = it },
                placeholder = {
                    Text(text = stringResource(R.string.import_override_textfield_placeholder))
                },
                colors = mullvadWhiteTextFieldColors(),
                textStyle =
                    MaterialTheme.typography.bodyLarge.copy(textDirection = TextDirection.Ltr),
            )
        }
    }
}

package net.mullvad.mullvadvpn.lib.feature.impl

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.Image
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SecureTextField
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldLabelPosition
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import java.time.Instant
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.animation.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.TunnelStats
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.ScreenDescription
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadSnackbar
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel

@Destination<ExternalModuleGraph>(style = SlideInFromRightTransition::class)
@Composable
fun PersonalVpn(navigator: DestinationsNavigator) {
    val vm = koinViewModel<PersonalVpnViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    val snackbarHostState = remember { SnackbarHostState() }

    CollectSideEffectWithLifecycle(vm.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            is PersonalVpnSideEffect.FailedToSave ->
                snackbarHostState.showSnackbarImmediately(sideEffect.reason)
        }
    }

    PersonalVpnScreen(
        state = state,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        vm::onToggle,
        vm::save,
        snackbarHostState = snackbarHostState,
    )
}

@Preview
@Composable
private fun PreviewPersonalVpnScreen() {
    AppTheme {
        PersonalVpnScreen(
            state = Lc.Content(PersonalVpnUiState(enabled = true, tunnelStats = TunnelStats())),
            onBackClick = {},
            onTogglePersonalVpn = {},
            saveConfig = {},
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PersonalVpnScreen(
    state: Lc<Boolean, PersonalVpnUiState>,
    onBackClick: () -> Unit,
    onTogglePersonalVpn: (Boolean) -> Unit,
    saveConfig: (PersonalVpnFormData) -> Unit,
    modifier: Modifier = Modifier,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.personal_vpn),
        modifier = modifier,
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier ->
        Column(
            modifier = modifier.animateContentSize().padding(horizontal = Dimens.sideMarginNew),
            verticalArrangement = Arrangement.spacedBy(Dimens.mediumPadding),
        ) {
            if (state is Lc.Content) {

                // Scale image to fit width up to certain width
                Image(
                    contentScale = ContentScale.FillWidth,
                    modifier =
                        Modifier.widthIn(max = Dimens.settingsDetailsImageMaxWidth)
                            .fillMaxWidth()
                            .align(Alignment.CenterHorizontally),
                    painter = painterResource(id = R.drawable.personal_vpn_illustration),
                    contentDescription = stringResource(R.string.multihop),
                )
                ScreenDescription(stringResource(R.string.personal_vpn_description))
                SwitchListItem(
                    title = stringResource(R.string.enable),
                    isToggled = state.value.enabled,
                    onCellClicked = { onTogglePersonalVpn(!state.value.enabled) },
                    position = Position.Single,
                )

                val tunnelStats = state.value.tunnelStats
                Text(modifier = Modifier.padding(horizontal = 16.dp), text = "Tunnel stats:")
                Column(
                    modifier =
                        Modifier.border(
                                1.dp,
                                MaterialTheme.colorScheme.primary,
                                RoundedCornerShape(16.dp),
                            )
                            .padding(8.dp)
                            .fillMaxWidth()
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Text("rx:")
                        Text("${tunnelStats.rx} bytes")
                    }
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Text("tx:")
                        Text("${tunnelStats.tx} bytes")
                    }
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Text("last handshake:")
                        val lastHandshakeSeconds = tunnelStats.lastHandshake?.epochSecond
                        val lastHandshakeString =
                            if (lastHandshakeSeconds == null) {
                                "Never"
                            } else {
                                val diff = Instant.now().epochSecond - lastHandshakeSeconds
                                "$diff seconds ago"
                            }
                        Text(lastHandshakeString)
                    }
                }

                val initialFormData = state.value.initialFormData
                val privateKeyTextFieldState = rememberTextFieldState(initialFormData.privateKey)
                val addressTextFieldState = rememberTextFieldState(initialFormData.tunnelIp)
                val publicKeyTextFieldState = rememberTextFieldState(initialFormData.publicKey)
                val allowedIpTextFieldState = rememberTextFieldState(initialFormData.allowedIP)
                val endpointTextFieldState = rememberTextFieldState(initialFormData.endpoint)

                Text("Interface")
                SecureTextField(
                    modifier = Modifier.fillMaxWidth(),
                    state = privateKeyTextFieldState,
                    label = { Text("Private key") },
                    labelPosition = TextFieldLabelPosition.Above(),
                    isError = state.value.privateKeyDataError != null,
                    placeholder = { Text("abcd...") },
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                    supportingText =
                        state.value.privateKeyDataError?.let { { Text(it.toErrorMessage()) } },
                )
                TextField(
                    modifier = Modifier.fillMaxWidth(),
                    state = addressTextFieldState,
                    label = { Text("Address") },
                    labelPosition = TextFieldLabelPosition.Above(),
                    placeholder = { Text("127.0.0.1") },
                    isError = state.value.tunnelIpDataError != null,
                    supportingText =
                        state.value.tunnelIpDataError?.let { { Text(it.toErrorMessage()) } },
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                )

                Spacer(modifier = Modifier.height(16.dp))
                Text("Peer")
                TextField(
                    modifier = Modifier.fillMaxWidth(),
                    state = publicKeyTextFieldState,
                    label = { Text("Public key") },
                    labelPosition = TextFieldLabelPosition.Above(),
                    placeholder = { Text("abcd...") },
                    isError = state.value.publicKeyDataError != null,
                    supportingText =
                        state.value.publicKeyDataError?.let { { Text(it.toErrorMessage()) } },
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                )
                TextField(
                    modifier = Modifier.fillMaxWidth(),
                    state = allowedIpTextFieldState,
                    label = { Text("Allowed IPs") },
                    labelPosition = TextFieldLabelPosition.Above(),
                    placeholder = { Text("10.0.0.0/8") },
                    isError = state.value.allowedIpDataError != null,
                    supportingText =
                        state.value.allowedIpDataError?.let { { Text(it.toErrorMessage()) } },
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
                )
                TextField(
                    modifier = Modifier.fillMaxWidth(),
                    state = endpointTextFieldState,
                    label = { Text("Endpoint") },
                    labelPosition = TextFieldLabelPosition.Above(),
                    placeholder = { Text("1.2.3.4:1234") },
                    isError = state.value.endpointDataError != null,
                    supportingText =
                        state.value.endpointDataError?.let { { Text(it.toErrorMessage()) } },
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
                    onKeyboardAction = {
                        saveConfig(
                            PersonalVpnFormData(
                                privateKey = privateKeyTextFieldState.text.toString(),
                                tunnelIp = addressTextFieldState.text.toString(),
                                publicKey = publicKeyTextFieldState.text.toString(),
                                allowedIP = allowedIpTextFieldState.text.toString(),
                                endpoint = endpointTextFieldState.text.toString(),
                            )
                        )
                    },
                )

                Spacer(modifier = Modifier.weight(1f))

                PrimaryButton(
                    text = stringResource(R.string.save),
                    onClick = {
                        saveConfig(
                            PersonalVpnFormData(
                                privateKey = privateKeyTextFieldState.text.toString(),
                                tunnelIp = addressTextFieldState.text.toString(),
                                publicKey = publicKeyTextFieldState.text.toString(),
                                allowedIP = allowedIpTextFieldState.text.toString(),
                                endpoint = endpointTextFieldState.text.toString(),
                            )
                        )
                    },
                )

                SnackbarHost(hostState = snackbarHostState) { MullvadSnackbar(snackbarData = it) }
            }
        }
    }
}

@Composable
fun FormDataError.toErrorMessage(): String =
    when (this) {
        FormDataError.AllowedIp -> "Bad allowed IP"
        is FormDataError.Endpoint -> this.toString()
        is FormDataError.PrivateKey -> keyParseError.toString()
        is FormDataError.PublicKey -> keyParseError.toString()
        FormDataError.TunnelIp -> "Bad address IP"
    }

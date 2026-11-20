package net.mullvad.mullvadvpn.feature.customvpn.impl.edit

import android.content.Context
import android.os.Build
import androidx.biometric.AuthenticationRequest
import androidx.biometric.AuthenticationRequest.Companion.biometricRequest
import androidx.biometric.AuthenticationResult
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_STRONG
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_WEAK
import androidx.biometric.BiometricManager.Authenticators.DEVICE_CREDENTIAL
import androidx.biometric.compose.rememberAuthenticationLauncher
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.TextObfuscationMode
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.SecureTextField
import androidx.compose.material3.TextFieldColors
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.TextFieldLabelScope
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.KeyboardType
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import net.mullvad.mullvadvpn.lib.ui.resource.R

@Composable
fun BiometricPasswordField(
    state: TextFieldState,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    textStyle: TextStyle = LocalTextStyle.current,
    label: @Composable (TextFieldLabelScope.() -> Unit)? = null,
    placeholder: @Composable (() -> Unit)? = null,
    supportingText: @Composable (() -> Unit)? = null,
    keyboardOptions: KeyboardOptions =
        KeyboardOptions(autoCorrectEnabled = false, keyboardType = KeyboardType.Password),
    colors: TextFieldColors = TextFieldDefaults.colors(),
    isError: Boolean = false,
    authTitle: String,
    authSubtitle: String,
) {
    val context = LocalContext.current
    var passwordVisible by rememberSaveable { mutableStateOf(false) }

    val authLauncher = rememberAuthenticationLauncher { result ->
        if (result is AuthenticationResult.Success) {
            passwordVisible = true
        }
    }

    LifecycleEventEffect(Lifecycle.Event.ON_STOP) { passwordVisible = false }

    Column(modifier = modifier) {
        SecureTextField(
            state = state,
            modifier = modifier,
            enabled = enabled,
            textStyle = textStyle,
            label = label,
            placeholder = placeholder,
            isError = isError,
            colors = colors,
            textObfuscationMode =
                if (passwordVisible) TextObfuscationMode.Visible
                else TextObfuscationMode.RevealLastTyped,
            keyboardOptions = keyboardOptions,
            supportingText = supportingText,
            trailingIcon = {
                val icon =
                    if (passwordVisible) {
                        Icons.Outlined.VisibilityOff
                    } else {
                        Icons.Outlined.Visibility
                    }
                val description =
                    if (passwordVisible) stringResource(R.string.hide_text)
                    else stringResource(R.string.show_text)

                IconButton(
                    onClick = {
                        if (passwordVisible) {
                            passwordVisible = false
                        } else if (canAuthenticate(context)) {
                            authLauncher.launch(
                                biometricRequest(
                                    title = authTitle,
                                    authFallbacks =
                                        arrayOf(
                                            AuthenticationRequest.Biometric.Fallback
                                                .DeviceCredential,
                                        ),
                                ) {
                                    setSubtitle(authSubtitle)
                                },
                            )
                        } else {
                            // Note: If canAuthenticate returns false we default to letting the user
                            // see the password.
                            passwordVisible = true
                        }
                    },
                ) {
                    Icon(imageVector = icon, contentDescription = description)
                }
            },
        )
    }
}

private fun canAuthenticate(context: Context): Boolean {
    val manager = BiometricManager.from(context)
    val authenticators =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            BIOMETRIC_STRONG or BIOMETRIC_WEAK or DEVICE_CREDENTIAL
        } else {
            BIOMETRIC_WEAK
        }
    return manager.canAuthenticate(authenticators) == BiometricManager.BIOMETRIC_SUCCESS
}

package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.material.Button
import androidx.compose.material.ButtonDefaults
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.textfield.MtuTextField
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite20
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite60
import net.mullvad.mullvadvpn.constant.MTU_MAX_VALUE
import net.mullvad.mullvadvpn.constant.MTU_MIN_VALUE
import net.mullvad.mullvadvpn.util.isValidMtu

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun MtuDialog(
    mtuValue: String,
    onMtuValueChanged: (String) -> Unit,
    onSave: () -> Unit,
    onRestoreDefaultValue: () -> Unit,
    onDismiss: () -> Unit,
) {
    val buttonSize = dimensionResource(id = R.dimen.button_height)
    val mediumPadding = dimensionResource(id = R.dimen.medium_padding)
    val dialogPadding = 20.dp
    val midPadding = 10.dp
    val smallPadding = 5.dp

    val textSmallSize = dimensionResource(id = R.dimen.text_small).value.sp
    val textMediumSize = dimensionResource(id = R.dimen.text_medium_plus).value.sp
    val textBigSize = dimensionResource(id = R.dimen.text_big).value.sp

    val textFieldFocusRequester = FocusRequester()
    val isValidMtu = mtuValue.toIntOrNull()?.isValidMtu() == true

    Dialog(
        // Fix for https://issuetracker.google.com/issues/221643630
        properties = DialogProperties(usePlatformDefaultWidth = false),
        onDismissRequest = { onDismiss() },
        content = {
            Column(
                Modifier
                    // Related to the fix for https://issuetracker.google.com/issues/221643630
                    .fillMaxWidth(0.8f)
                    .background(color = MullvadDarkBlue)
                    .padding(dialogPadding)
            ) {
                Text(
                    text = stringResource(id = R.string.wireguard_mtu),
                    color = Color.White,
                    fontSize = textBigSize
                )

                Box(
                    Modifier.wrapContentSize().clickable { textFieldFocusRequester.requestFocus() }
                ) {
                    MtuTextField(
                        value = mtuValue,
                        onValueChanged = { newMtuValue -> onMtuValueChanged(newMtuValue) },
                        onFocusChange = {},
                        onSubmit = { newMtuValue ->
                            if (newMtuValue.toIntOrNull()?.isValidMtu() == true) {
                                onSave()
                            }
                        },
                        isEnabled = true,
                        placeholderText = stringResource(R.string.enter_value_placeholder),
                        maxCharLength = 4,
                        isValidValue = isValidMtu,
                        modifier =
                            Modifier.padding(top = midPadding)
                                .focusRequester(textFieldFocusRequester)
                    )
                }

                Text(
                    text =
                        stringResource(
                            id = R.string.wireguard_mtu_footer,
                            MTU_MIN_VALUE,
                            MTU_MAX_VALUE
                        ),
                    fontSize = textSmallSize,
                    color = MullvadWhite60,
                    modifier = Modifier.padding(top = smallPadding)
                )

                Button(
                    modifier =
                        Modifier.padding(top = mediumPadding).height(buttonSize).fillMaxWidth(),
                    colors =
                        ButtonDefaults.buttonColors(
                            backgroundColor = MullvadBlue,
                            contentColor = MullvadWhite,
                            disabledContentColor = MullvadWhite60,
                            disabledBackgroundColor = MullvadWhite20
                        ),
                    enabled = isValidMtu,
                    onClick = { onSave() }
                ) {
                    Text(text = stringResource(R.string.submit_button), fontSize = textMediumSize)
                }

                Button(
                    modifier =
                        Modifier.padding(top = mediumPadding)
                            .height(buttonSize)
                            .defaultMinSize(minHeight = buttonSize)
                            .fillMaxWidth(),
                    colors =
                        ButtonDefaults.buttonColors(
                            backgroundColor = MullvadBlue,
                            contentColor = MullvadWhite
                        ),
                    onClick = { onRestoreDefaultValue() }
                ) {
                    Text(
                        text = stringResource(R.string.reset_to_default_button),
                        fontSize = textMediumSize
                    )
                }

                Button(
                    modifier =
                        Modifier.padding(top = mediumPadding)
                            .height(buttonSize)
                            .defaultMinSize(minHeight = buttonSize)
                            .fillMaxWidth(),
                    colors =
                        ButtonDefaults.buttonColors(
                            backgroundColor = MullvadBlue,
                            contentColor = Color.White
                        ),
                    onClick = { onDismiss() }
                ) {
                    Text(text = stringResource(R.string.cancel), fontSize = textMediumSize)
                }
            }
        }
    )
}

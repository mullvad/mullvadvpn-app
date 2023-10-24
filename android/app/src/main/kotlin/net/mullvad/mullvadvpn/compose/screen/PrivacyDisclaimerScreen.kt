package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.ClickableText
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewPrivacyDisclaimerScreen() {
    AppTheme { PrivacyDisclaimerScreen({}, {}) }
}

@Composable
fun PrivacyDisclaimerScreen(
    onPrivacyPolicyLinkClicked: () -> Unit,
    onAcceptClicked: () -> Unit,
) {
    val topColor = MaterialTheme.colorScheme.primary
    ScaffoldWithTopBar(
        topBarColor = topColor,
        statusBarColor = topColor,
        navigationBarColor = MaterialTheme.colorScheme.background,
        onAccountClicked = null,
        onSettingsClicked = null
    ) {
        ConstraintLayout(
            modifier =
                Modifier.fillMaxHeight()
                    .fillMaxWidth()
                    .padding(it)
                    .background(color = MaterialTheme.colorScheme.background)
        ) {
            val (body, actionButtons) = createRefs()
            val sideMargin = Dimens.sideMargin

            Column(
                modifier =
                    Modifier.padding(bottom = Dimens.privacyPolicyBottomPadding)
                        .verticalScroll(rememberScrollState())
                        .constrainAs(body) {
                            top.linkTo(parent.top, margin = sideMargin)
                            start.linkTo(parent.start, margin = sideMargin)
                            end.linkTo(parent.end, margin = sideMargin)
                            width = Dimension.fillToConstraints
                        },
            ) {
                Text(
                    text = stringResource(id = R.string.privacy_disclaimer_title),
                    fontSize = 24.sp,
                    color = Color.White,
                    fontWeight = FontWeight.Bold
                )

                Text(
                    text = stringResource(id = R.string.privacy_disclaimer_body),
                    fontSize = 14.sp,
                    color = Color.White,
                    modifier = Modifier.padding(top = 10.dp)
                )

                Row(modifier = Modifier.padding(top = 10.dp)) {
                    ClickableText(
                        text = AnnotatedString(stringResource(id = R.string.privacy_policy_label)),
                        onClick = { onPrivacyPolicyLinkClicked.invoke() },
                        style =
                            TextStyle(
                                fontSize = 12.sp,
                                color = Color.White,
                                textDecoration = TextDecoration.Underline
                            )
                    )

                    Image(
                        painter = painterResource(id = R.drawable.icon_extlink),
                        contentDescription = null,
                        modifier =
                            Modifier.align(Alignment.CenterVertically)
                                .padding(start = 2.dp, top = 2.dp)
                                .width(10.dp)
                                .height(10.dp)
                    )
                }
            }

            Column(
                modifier =
                    Modifier.constrainAs(actionButtons) {
                        bottom.linkTo(parent.bottom, margin = sideMargin)
                        start.linkTo(parent.start, margin = sideMargin)
                        end.linkTo(parent.end, margin = sideMargin)
                        width = Dimension.fillToConstraints
                    }
            ) {
                PrimaryButton(
                    text = stringResource(id = R.string.agree_and_continue),
                    onClick = onAcceptClicked::invoke
                )
            }
        }
    }
}

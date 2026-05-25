package net.mullvad.mullvadvpn.feature.customlist.impl.screen.editlocations

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.clearText
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Clear
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.tooling.preview.Preview
import kotlinx.coroutines.flow.collectLatest
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.Alpha10

@Preview
@Composable
private fun PreviewSearchTextField() {
    AppTheme {
        Column(modifier = Modifier.background(color = MaterialTheme.colorScheme.background)) {
            SearchTextField(
                placeHolder = "Search for...",
                backgroundColor = MaterialTheme.colorScheme.onSurface.copy(alpha = Alpha10),
                textColor = MaterialTheme.colorScheme.onTertiaryContainer,
            ) {}
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SearchTextField(
    modifier: Modifier = Modifier,
    placeHolder: String = stringResource(id = R.string.search_placeholder),
    backgroundColor: Color,
    textColor: Color,
    enabled: Boolean = true,
    singleLine: Boolean = true,
    interactionSource: MutableInteractionSource = remember { MutableInteractionSource() },
    visualTransformation: VisualTransformation = VisualTransformation.None,
    onValueChange: (String) -> Unit,
) {
    val textFieldState = rememberTextFieldState("")
    LaunchedEffect(textFieldState) {
        snapshotFlow { textFieldState.text.toString() }.collectLatest { onValueChange(it) }
    }

    BasicTextField(
        state = textFieldState,
        textStyle = MaterialTheme.typography.bodyLarge.copy(color = textColor),
        lineLimits =
            if (singleLine) {
                TextFieldLineLimits.SingleLine
            } else {
                TextFieldLineLimits.MultiLine()
            },
        cursorBrush = SolidColor(textColor),
        decorator =
            @Composable { innerTextField ->
                TextFieldDefaults.DecorationBox(
                    value = textFieldState.text.toString(),
                    innerTextField = innerTextField,
                    enabled = enabled,
                    singleLine = singleLine,
                    interactionSource = interactionSource,
                    visualTransformation = visualTransformation,
                    leadingIcon = {
                        Icon(
                            imageVector = Icons.Rounded.Search,
                            contentDescription = null,
                            modifier =
                                Modifier.size(
                                    width = Dimens.searchIconSize,
                                    height = Dimens.searchIconSize,
                                ),
                            tint = textColor,
                        )
                    },
                    placeholder = {
                        Text(text = placeHolder, style = MaterialTheme.typography.bodyLarge)
                    },
                    trailingIcon = {
                        if (textFieldState.text.isNotEmpty()) {
                            Icon(
                                modifier =
                                    Modifier.size(Dimens.smallIconSize).clickable {
                                        textFieldState.clearText()
                                        onValueChange.invoke(textFieldState.text.toString())
                                    },
                                imageVector = Icons.Rounded.Clear,
                                tint = textColor,
                                contentDescription = null,
                            )
                        }
                    },
                    shape = MaterialTheme.shapes.medium,
                    colors =
                        TextFieldDefaults.colors(
                            focusedTextColor = textColor,
                            unfocusedTextColor = textColor,
                            focusedContainerColor = backgroundColor,
                            unfocusedContainerColor = backgroundColor,
                            focusedIndicatorColor = Color.Transparent,
                            unfocusedIndicatorColor = Color.Transparent,
                            cursorColor = textColor,
                            focusedPlaceholderColor = textColor,
                            unfocusedPlaceholderColor = textColor,
                        ),
                    contentPadding = PaddingValues(),
                )
            },
        modifier = modifier,
    )
}

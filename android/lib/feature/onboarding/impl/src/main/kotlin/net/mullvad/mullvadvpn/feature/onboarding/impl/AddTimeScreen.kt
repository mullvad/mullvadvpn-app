package net.mullvad.mullvadvpn.feature.onboarding.impl

import androidx.activity.ComponentActivity
import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.addtime.api.AddTimeNavKey
import net.mullvad.mullvadvpn.feature.addtime.api.VerificationPendingNavKey
import net.mullvad.mullvadvpn.feature.home.api.ConnectNavKey
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingNavKey
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingStep
import net.mullvad.mullvadvpn.lib.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.ui.designsystem.VariantButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.ONBOARDING_ADD_TIME_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewAddTimeScreen() {
    AppTheme {
        AddTimeScreen(
            state = AddTimeUiState(paymentStatus = PaymentStatus.PENDING),
            onBackClick = {},
            onAddTimeClick = {},
            onPaymentInfoClick = {},
        )
    }
}

@Composable
@Suppress("LongMethod")
fun AddTime(navKey: OnboardingNavKey, navigator: Navigator) {
    val activity = LocalActivity.current as ComponentActivity
    val vm: AddTimeViewModel = koinViewModel { parametersOf(activity.lifecycle) }
    val state by vm.uiState.collectAsStateWithLifecycle()

    var timeAdded by rememberSaveable { mutableStateOf(false) }
    var onboardingAlreadyCompleted by rememberSaveable { mutableStateOf(false) }

    CollectSideEffectWithLifecycle(sideEffect = vm.uiSideEffect) { effect ->
        when (effect) {
            is AddTimeViewModel.UiSideEffect.TimeAdded -> {
                timeAdded = true
                onboardingAlreadyCompleted = effect.hasCompletedOnboarding
            }
        }
    }

    val isTopOfBackStack = navigator.state.backStack.lastOrNull() == navKey
    LaunchedEffect(timeAdded, isTopOfBackStack) {
        if (!timeAdded || !isTopOfBackStack) {
            return@LaunchedEffect
        }
        // Consume the event before navigating so a recomposition cannot navigate twice.
        timeAdded = false
        val destination =
            if (onboardingAlreadyCompleted) {
                ConnectNavKey
            } else {
                OnboardingNavKey(OnboardingStep.Notifications)
            }
        navigator.navigate(destination, clearBackStack = true)
    }

    AddTimeScreen(
        state = state,
        onBackClick = dropUnlessResumed { navigator.goBack() },
        onAddTimeClick = dropUnlessResumed { navigator.navigate(AddTimeNavKey) },
        onPaymentInfoClick =
            dropUnlessResumed { paymentStatus ->
                navigator.navigate(VerificationPendingNavKey(paymentStatus))
            },
    )
}

@Composable
fun AddTimeScreen(
    state: AddTimeUiState,
    onPaymentInfoClick: (PaymentStatus) -> Unit,
    onBackClick: () -> Unit,
    onAddTimeClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    OnboardingScaffold(
        modifier = modifier,
        onBackClick = onBackClick,
        bottomBar = {
            OnboardingBottomBar {
                if (state.paymentStatus != null) {
                    PaymentPending(
                        onPaymentInfoClick = onPaymentInfoClick,
                        paymentStatus = state.paymentStatus,
                    )
                }
                VariantButton(
                    onClick = onAddTimeClick,
                    text = stringResource(id = R.string.add_time),
                    modifier = Modifier.fillMaxWidth().testTag(ONBOARDING_ADD_TIME_BUTTON_TEST_TAG),
                )
            }
        },
    ) {
        Text(
            text = stringResource(id = R.string.add_time),
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.fillMaxWidth(),
        )
        OnboardingBodyText(
            text = stringResource(id = R.string.onboarding_add_time_first_paragraph),
        )
        OnboardingBodyText(
            text = stringResource(id = R.string.onboarding_add_time_second_paragraph),
        )
        OnboardingBodyText(text = stringResource(id = R.string.onboarding_add_time_third_paragraph))
    }
}

@Composable
private fun PaymentPending(
    onPaymentInfoClick: (PaymentStatus) -> Unit,
    paymentStatus: PaymentStatus,
) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        IconButton(
            onClick = { onPaymentInfoClick(paymentStatus) },
            modifier = Modifier.testTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG),
        ) {
            Icon(
                imageVector = Icons.Rounded.Info,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurface,
            )
        }
        Text(
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurface,
            text =
                stringResource(
                    id =
                        when (paymentStatus) {
                            PaymentStatus.PENDING -> R.string.payment_status_pending_short

                            PaymentStatus.PURCHASED_UNVERIFIED ->
                                R.string.payment_status_verification_failed
                        },
                ),
        )
    }
}

package net.mullvad.mullvadvpn.feature.onboarding.impl

import android.Manifest
import android.os.Build
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.PermissionStatus
import com.google.accompanist.permissions.rememberPermissionState
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.addtime.api.AddTimeNavKey
import net.mullvad.mullvadvpn.feature.home.api.ConnectNavKey
import net.mullvad.mullvadvpn.feature.onboarding.api.GoodToKnowStep
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingConnectNavResult
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingNavKey
import net.mullvad.mullvadvpn.feature.onboarding.api.OnboardingStep
import net.mullvad.mullvadvpn.lib.common.compose.CollectSideEffectWithLifecycle
import org.koin.compose.viewmodel.koinActivityViewModel

@OptIn(ExperimentalPermissionsApi::class)
@Composable
@Suppress("LongMethod", "CyclomaticComplexMethod")
fun Onboarding(navKey: OnboardingNavKey, navigator: Navigator) {
    val vm: OnboardingViewModel = koinActivityViewModel()
    val step = navKey.step
    val state by vm.uiState.collectAsStateWithLifecycle()
    val resultStore = LocalResultStore.current

    LaunchedEffect(step) { vm.onStepEntered(step) }

    var timeAdded by remember { mutableStateOf<OnboardingViewModel.UiSideEffect.TimeAdded?>(null) }

    // If time was added, only navigate when this screen is at the top of the backstack
    // (all add time popups have been dismissed by the user).
    if (
        timeAdded != null &&
            navigator.state.backStack.last() == navKey &&
            step == OnboardingStep.AddTime
    ) {
        val onboardingCompleted = timeAdded?.hasCompletedOnboarding ?: false
        val destination =
            if (onboardingCompleted) {
                ConnectNavKey
            } else {
                OnboardingNavKey(OnboardingStep.Notifications)
            }
        navigator.navigate(destination, clearBackStack = true)
    }

    CollectSideEffectWithLifecycle(sideEffect = vm.uiSideEffect) { effect ->
        when (effect) {
            is OnboardingViewModel.UiSideEffect.TimeAdded -> timeAdded = effect
        }
    }

    fun finish(connect: Boolean) {
        vm.complete()
        if (connect) {
            resultStore.setResult(OnboardingConnectNavResult)
        }
        navigator.navigate(ConnectNavKey, clearBackStack = true)
    }

    fun goToGoodToKnow() {
        vm.onNotificationStepDismissed()
        navigator.navigate(OnboardingNavKey(GoodToKnowStep.GoodToKnow1), clearBackStack = true)
    }

    val notificationPermissionState =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            rememberPermissionState(permission = Manifest.permission.POST_NOTIFICATIONS) {
                goToGoodToKnow()
            }
        } else {
            null
        }

    when (step) {
        OnboardingStep.AddTime -> {
            AddTimeOnboardingScreen(
                onBackClick = dropUnlessResumed { navigator.goBack() },
                onAddTimeClick = dropUnlessResumed { navigator.navigate(AddTimeNavKey) },
            )
        }
        OnboardingStep.Notifications -> {
            if (
                Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
                    notificationPermissionState?.status == PermissionStatus.Granted
            ) {
                LaunchedEffect(Unit) { goToGoodToKnow() }
            } else {
                NotificationsOnboardingScreen(
                    onEnableNotificationsClick =
                        dropUnlessResumed {
                            notificationPermissionState?.launchPermissionRequest()
                        },
                    onNextClick = dropUnlessResumed { goToGoodToKnow() },
                )
            }
        }
        is GoodToKnowStep -> {
            GoodToKnowScreen(
                step = step,
                onBackClick = dropUnlessResumed { navigator.goBack() },
                onNextClick =
                    dropUnlessResumed {
                        val next =
                            if (step == GoodToKnowStep.GoodToKnow1) {
                                GoodToKnowStep.GoodToKnow2
                            } else {
                                OnboardingStep.ReadyToConnect
                            }
                        navigator.navigate(OnboardingNavKey(next))
                    },
                onSkipClick =
                    dropUnlessResumed { navigator.navigate(ConnectNavKey, clearBackStack = true) },
            )
        }
        OnboardingStep.ReadyToConnect -> {
            ReadyToConnectScreen(
                selectedLocationTitle = state.selectedLocationTitle,
                onBackClick = dropUnlessResumed { navigator.goBack() },
                onConnectClick = dropUnlessResumed { finish(connect = true) },
                onLaterClick = dropUnlessResumed { finish(connect = false) },
            )
        }
    }
}

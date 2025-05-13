package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.PaymentDestination
import com.ramcosta.composedestinations.generated.destinations.RedeemVoucherDestination
import com.ramcosta.composedestinations.generated.destinations.VerificationPendingDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.RedeemVoucherButton
import net.mullvad.mullvadvpn.compose.button.SitePaymentButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.PlayPayment
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.extensions.createOpenAccountPageHook
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.AddMoreTimeUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.AddMoreTimeUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.util.Lce
import net.mullvad.mullvadvpn.viewmodel.AddMoreTimeSideEffect
import net.mullvad.mullvadvpn.viewmodel.AddMoreTimeViewModel
import org.koin.androidx.compose.koinViewModel

@Composable
@Preview(
    "Loading|oss|LoadingSitePayment|" +
        "PaymentLoading|NoPayment|NoProductsFound|PaymentAvailable|PaymentPending|PaymentError"
)
fun AddMoreTimePreview(
    @PreviewParameter(AddMoreTimeUiStatePreviewParameterProvider::class)
    state: Lce<Unit, AddMoreTimeUiState, Unit>
) {
    AppTheme {
        AddMoreTimeScreen(
            state = state,
            onPurchaseBillingProductClick = {},
            onPlayPaymentInfoClick = {},
            onSitePaymentClick = {},
            onRedeemVoucherClick = {},
            onBackClick = {},
        )
    }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun AddMoreTime(
    navigator: DestinationsNavigator,
    playPaymentResultRecipient: ResultRecipient<PaymentDestination, Boolean>,
) {
    val viewModel = koinViewModel<AddMoreTimeViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    val openAccountPage = LocalUriHandler.current.createOpenAccountPageHook()
    CollectSideEffectWithLifecycle(
        viewModel.uiSideEffect,
        minActiveState = Lifecycle.State.RESUMED,
    ) { sideEffect ->
        when (sideEffect) {
            is AddMoreTimeSideEffect.OpenAccountManagementPageInBrowser -> openAccountPage
            is AddMoreTimeSideEffect.GenericError -> {}
        }
    }

    playPaymentResultRecipient.OnNavResultValue { viewModel.onClosePurchaseResultDialog(it) }

    AddMoreTimeScreen(
        state = state,
        onPurchaseBillingProductClick =
            dropUnlessResumed { productId -> navigator.navigate(PaymentDestination(productId)) },
        onPlayPaymentInfoClick =
            dropUnlessResumed { navigator.navigate(VerificationPendingDestination) },
        onSitePaymentClick = viewModel::onManageAccountClick,
        onRedeemVoucherClick = dropUnlessResumed { navigator.navigate(RedeemVoucherDestination) },
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
fun AddMoreTimeScreen(
    state: Lce<Unit, AddMoreTimeUiState, Unit>,
    onPurchaseBillingProductClick: (ProductId) -> Unit,
    onPlayPaymentInfoClick: () -> Unit,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(R.string.add_more_time_screen_title),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier: Modifier, lazyListState: LazyListState ->
        LazyColumn(
            modifier =
                modifier
                    .padding(horizontal = Dimens.sideMargin)
                    .padding(bottom = Dimens.screenBottomMargin),
            state = lazyListState,
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(space = Dimens.buttonSpacing),
        ) {
            when (state) {
                is Lce.Loading -> loading()
                is Lce.Content ->
                    content(
                        state.value,
                        onPurchaseBillingProductClick,
                        onPlayPaymentInfoClick,
                        onSitePaymentClick,
                        onRedeemVoucherClick,
                    )
                is Lce.Error -> error()
            }
        }
    }
}

private fun LazyListScope.content(
    state: AddMoreTimeUiState,
    onPurchaseBillingProductClick: (ProductId) -> Unit,
    onPlayPaymentInfoClick: () -> Unit,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
) {
    state.billingPaymentState?.let {
        item {
            PlayPayment(
                modifier = Modifier.fillMaxWidth(),
                billingPaymentState = state.billingPaymentState,
                onPurchaseBillingProductClick = onPurchaseBillingProductClick,
                onInfoClick = onPlayPaymentInfoClick,
            )
        }
    }
    if (state.showSitePayment) {
        item {
            SitePaymentButton(
                modifier = Modifier.padding(bottom = Dimens.buttonSpacing),
                isLoading = state.showManageAccountLoading,
                onClick = onSitePaymentClick,
                isEnabled = true,
            )
        }
    }
    item {
        RedeemVoucherButton(
            onClick = onRedeemVoucherClick,
            modifier = Modifier.padding(bottom = Dimens.buttonSpacing),
            isEnabled = true,
        )
    }
}

private fun LazyListScope.loading() {
    item(contentType = ContentType.PROGRESS) { MullvadCircularProgressIndicatorLarge() }
}

// TODO Do we need this?
private fun LazyListScope.error() {
    item(contentType = ContentType.PROGRESS) { MullvadCircularProgressIndicatorLarge() }
}

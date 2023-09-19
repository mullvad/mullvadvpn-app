package net.mullvad.mullvadvpn.di

import net.mullvad.mullvadvpn.PaymentProvider
import net.mullvad.mullvadvpn.lib.billing.BillingPaymentRepository
import net.mullvad.mullvadvpn.lib.billing.BillingRepository
import net.mullvad.mullvadvpn.lib.billing.PlayPurchaseRepository
import org.koin.android.ext.koin.androidContext
import org.koin.dsl.module

val paymentModule = module {
    single { BillingRepository(androidContext()) }
    single { PaymentProvider(BillingPaymentRepository(get(), get())) }
    single { PlayPurchaseRepository(get()) }
}

package net.mullvad.mullvadvpn.di

import net.mullvad.mullvadvpn.lib.payment.PaymentProvider
import org.koin.dsl.module

val paymentModule = module { single { PaymentProvider(null) } }

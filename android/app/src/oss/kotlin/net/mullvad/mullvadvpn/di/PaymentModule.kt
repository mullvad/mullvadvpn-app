package net.mullvad.mullvadvpn.di

import net.mullvad.mullvadvpn.PaymentProvider
import org.koin.dsl.module

val paymentModule = module { single { PaymentProvider(null) } }

package net.mullvad.mullvadvpn.di

import net.mullvad.mullvadvpn.PaymentProvider
import net.mullvad.mullvadvpn.lib.payment.PaymentRepository
import org.koin.dsl.module

val paymentModule = module { single { PaymentProvider(null) } }

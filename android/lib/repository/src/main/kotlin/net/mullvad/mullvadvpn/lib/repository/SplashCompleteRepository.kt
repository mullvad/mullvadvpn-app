package net.mullvad.mullvadvpn.lib.repository

import dev.zacsweers.metro.AppScope
import dev.zacsweers.metro.ContributesBinding
import dev.zacsweers.metro.Inject
import dev.zacsweers.metro.SingleIn

//@ContributesBinding(AppScope::class)
@SingleIn(AppScope::class)
@Inject
class SplashCompleteRepository {
    private var splashComplete = false

    fun isSplashComplete() = splashComplete

    fun onSplashCompleted() {
        splashComplete = true
    }
}

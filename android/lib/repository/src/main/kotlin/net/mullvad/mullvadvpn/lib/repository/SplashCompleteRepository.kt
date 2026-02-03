package net.mullvad.mullvadvpn.lib.repository

class SplashCompleteRepository {
    private var splashComplete = false

    fun isSplashComplete() = splashComplete

    fun onSplashCompleted() {
        splashComplete = true
    }
}

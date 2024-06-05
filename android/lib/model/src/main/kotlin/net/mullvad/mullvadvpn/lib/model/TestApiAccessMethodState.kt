package net.mullvad.mullvadvpn.lib.model

sealed interface TestApiAccessMethodState {
    data object Testing : TestApiAccessMethodState

    sealed interface Result : TestApiAccessMethodState {
        data object Successful : Result

        data object Failure : Result

        fun isSuccessful() = this is Successful
    }
}

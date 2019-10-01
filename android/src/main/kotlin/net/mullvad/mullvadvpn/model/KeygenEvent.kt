package net.mullvad.mullvadvpn.model

sealed class KeygenEvent {
    class NewKey(
        val publicKey: PublicKey,
        val verified: Boolean?,
        val replacementFailure: KeygenFailure?
    ) : KeygenEvent()
    class Failure(val failure: KeygenFailure) : KeygenEvent()
}

sealed class KeygenFailure {
    class TooManyKeys : KeygenFailure()
    class GenerationFailure : KeygenFailure()
}

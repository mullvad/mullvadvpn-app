package net.mullvad.mullvadvpn.model

sealed class KeygenEvent {
    class NewKey(val publicKey: PublicKey) : KeygenEvent() {
        var verified: Boolean? = false
            private set
        var replacementFailure: KeygenFailure? = null
            private set

        constructor(
            publicKey: PublicKey,
            verified: Boolean?,
            replacementFailure: KeygenFailure?
        ) : this(publicKey) {
            this.verified = verified
            this.replacementFailure = replacementFailure
        }
    }
    class Failure(val failure: KeygenFailure) : KeygenEvent()
}

sealed class KeygenFailure {
    class TooManyKeys : KeygenFailure()
    class GenerationFailure : KeygenFailure()
}

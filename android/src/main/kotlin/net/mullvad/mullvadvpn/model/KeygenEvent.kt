package net.mullvad.mullvadvpn.model

sealed class KeygenEvent {
    class NewKey(val publicKey: PublicKey) : KeygenEvent() {
        var verified: Boolean? = null
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
    class TooManyKeys : KeygenEvent()
    class GenerationFailure : KeygenEvent()

    fun failure(): KeygenFailure? {
        return when (this) {
            is KeygenEvent.TooManyKeys -> KeygenFailure.TooManyKeys
            is KeygenEvent.GenerationFailure -> KeygenFailure.GenerationFailure
            else -> null
        }
    }
}

enum class KeygenFailure {
    TooManyKeys,
    GenerationFailure,
}

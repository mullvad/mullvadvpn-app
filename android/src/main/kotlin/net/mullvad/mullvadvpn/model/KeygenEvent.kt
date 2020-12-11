package net.mullvad.mullvadvpn.model

sealed class KeygenEvent {
    class NewKey(
        val publicKey: PublicKey,
        val verified: Boolean?,
        val replacementFailure: KeygenFailure?
    ) : KeygenEvent() {
        constructor(publicKey: PublicKey) : this (publicKey, null, null)
    }

    class TooManyKeys : KeygenEvent()
    class GenerationFailure : KeygenEvent()

    fun failure(): KeygenFailure? {
        return when (this) {
            is KeygenEvent.TooManyKeys -> KeygenFailure.TooManyKeys()
            is KeygenEvent.GenerationFailure -> KeygenFailure.GenerationFailure()
            else -> { null }
        }
    }
}

sealed class KeygenFailure() {
    class TooManyKeys() : KeygenFailure()
    class GenerationFailure() : KeygenFailure()
}

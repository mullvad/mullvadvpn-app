package net.mullvad.mullvadvpn.model

sealed class KeygenEvent {
    class NewKey(
        val publicKey: PublicKey,
        val verified: Boolean?,
        val replacementFailure: KeygenFailure?
    ) : KeygenEvent() {
        constructor(publicKey: PublicKey) : this (publicKey, null, null)
    }

    object TooManyKeys : KeygenEvent()
    object GenerationFailure : KeygenEvent()

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

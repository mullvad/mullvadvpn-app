package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class KeygenEvent : Parcelable {
    @Parcelize
    class NewKey(
        val publicKey: PublicKey,
        val verified: Boolean?,
        val replacementFailure: KeygenFailure?
    ) : KeygenEvent() {
        constructor(publicKey: PublicKey) : this (publicKey, null, null)
    }

    @Parcelize
    object TooManyKeys : KeygenEvent()

    @Parcelize
    object GenerationFailure : KeygenEvent()

    fun failure(): KeygenFailure? {
        return when (this) {
            is KeygenEvent.TooManyKeys -> KeygenFailure.TooManyKeys
            is KeygenEvent.GenerationFailure -> KeygenFailure.GenerationFailure
            else -> null
        }
    }
}

@Parcelize
enum class KeygenFailure : Parcelable {
    TooManyKeys,
    GenerationFailure,
}

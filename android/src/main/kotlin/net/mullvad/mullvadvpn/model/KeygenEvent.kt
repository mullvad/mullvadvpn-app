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
    class TooManyKeys : KeygenEvent(), Parcelable

    @Parcelize
    class GenerationFailure : KeygenEvent(), Parcelable

    fun failure(): KeygenFailure? {
        return when (this) {
            is KeygenEvent.TooManyKeys -> KeygenFailure.TooManyKeys()
            is KeygenEvent.GenerationFailure -> KeygenFailure.GenerationFailure()
            else -> { null }
        }
    }
}

sealed class KeygenFailure() : Parcelable {
    @Parcelize
    @Suppress("PARCELABLE_PRIMARY_CONSTRUCTOR_IS_EMPTY")
    class TooManyKeys() : KeygenFailure(), Parcelable

    @Parcelize
    @Suppress("PARCELABLE_PRIMARY_CONSTRUCTOR_IS_EMPTY")
    class GenerationFailure() : KeygenFailure(), Parcelable
}

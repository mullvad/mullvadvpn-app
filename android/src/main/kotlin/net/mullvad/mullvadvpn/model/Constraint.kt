package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class Constraint<T>() : Parcelable {
    @Parcelize
    @Suppress("PARCELABLE_PRIMARY_CONSTRUCTOR_IS_EMPTY")
    class Any<T>() : Constraint<T>(), Parcelable

    @Parcelize
    class Only<T : Parcelable>(val value: T) : Constraint<T>(), Parcelable {
        fun get0() = value
    }
}

package net.mullvad.mullvadvpn.core

sealed interface Lce<out L, out T, out E> {
    data class Loading<L>(val value: L) : Lce<L, Nothing, Nothing>

    data class Content<T>(val value: T) : Lce<Nothing, T, Nothing>

    data class Error<E>(val error: E) : Lce<Nothing, Nothing, E>

    fun contentOrNull(): T? = (this as? Content<T>)?.value

    fun errorOrNull(): E? = (this as? Error<E>)?.error
}

fun <L, T, E> T.toLce(): Lce<L, T, E> = Lce.Content(this)

sealed interface Lc<out L, out T> {
    data class Loading<L>(val value: L) : Lc<L, Nothing>

    data class Content<T>(val value: T) : Lc<Nothing, T>

    fun contentOrNull(): T? = (this as? Content<T>)?.value
}

fun <L, T> T.toLc(): Lc<L, T> = Lc.Content(this)

package net.mullvad.mullvadvpn.util

sealed interface Lce<out T, out E> {
    data object Loading : Lce<Nothing, Nothing>

    data class Content<T>(val value: T) : Lce<T, Nothing>

    data class Error<E>(val error: E) : Lce<Nothing, E>

    fun contentOrNull(): T? =
        when (this) {
            is Loading,
            is Error -> null
            is Content -> value
        }

    fun errorOrNull(): E? =
        when (this) {
            is Loading,
            is Content -> null
            is Error -> error
        }
}

fun <T, E> T.toLce(): Lce<T, E> = Lce.Content(this)

sealed interface Lc<out T> {
    data object Loading : Lc<Nothing>

    data class Content<T>(val value: T) : Lc<T>

    fun contentOrNull(): T? =
        when (this) {
            is Content -> value
            Loading -> null
        }
}

fun <T> T.toLc(): Lc<T> = Lc.Content(this)

package net.mullvad.mullvadvpn.util

sealed interface Lce<out T, out E> {
    data object Loading : Lce<Nothing, Nothing>

    data class Content<T>(val value: T) : Lce<T, Nothing>

    data class Error<E>(val error: E) : Lce<Nothing, E>

    fun content(): T? =
        when (this) {
            is Loading,
            is Error -> null
            is Content -> value
        }

    fun error(): E? =
        when (this) {
            is Loading,
            is Content -> null
            is Error -> error
        }

    fun isLoading(): Boolean = this is Loading
}

fun <T, E> T.toLce(): Lce<T, E> = Lce.Content(this)

sealed interface Lc<out T> {
    data object Loading : Lc<Nothing>

    data class Content<T>(val value: T) : Lc<T>

    fun content(): T? =
        when (this) {
            is Content -> value
            Loading -> null
        }

    fun isLoading(): Boolean = this is Loading
}

fun <T> T.toLc(): Lc<T> = Lc.Content(this)

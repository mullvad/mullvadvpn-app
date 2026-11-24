package net.mullvad.mullvadvpn.lib.common.util

import arrow.core.Either
import arrow.core.Either.Right
import arrow.core.left

inline fun <T, reified E : T> List<T>.getFirstInstanceOrNull(): E? =
    this.filterIsInstance<E>().firstOrNull()

@Suppress("LongParameterList")
inline fun <E1, E2, E3, E4, A, B, C, D, E, Z> zipFold(
    a: Either<E1, A>,
    b: Either<E2, B>,
    c: Either<E3, C>,
    d: Either<E4, D>,
    transformError: (E1?, E2?, E3?, E4?) -> E,
    transform: (A, B, C, D) -> Z,
): Either<E, Z> =
    if (a is Right && b is Right && c is Right && d is Right) {
        Right(transform(a.value, b.value, c.value, d.value))
    } else {
        transformError(a.leftOrNull(), b.leftOrNull(), c.leftOrNull(), d.leftOrNull()).left()
    }

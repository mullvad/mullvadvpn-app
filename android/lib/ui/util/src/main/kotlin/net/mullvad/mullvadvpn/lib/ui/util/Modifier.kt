import androidx.compose.ui.Modifier

fun <T> Modifier.ifNotNull(
    value: T?,
    and: Boolean = true,
    block: Modifier.(T) -> Modifier,
): Modifier =
    if (value != null && and) {
        then(Modifier.block(value))
    } else {
        this
    }

fun Modifier.ifTrue(condition: Boolean, modifier: Modifier): Modifier =
    if (condition) {
        then(modifier)
    } else {
        this
    }

package utilities

object BuildTypes {
    const val DEBUG = "debug"
    const val RELEASE = "release"
    const val FDROID = "fdroid"
    const val LEAK_CANARY = "leakCanary"

    const val NON_MINIFIED = "nonMinified"

    const val BENCHMARK = "benchmark"
}

object SigningConfigs {
    const val RELEASE = "release"
}

object FlavorDimensions {
    const val BILLING = "billing"
    const val INFRASTRUCTURE = "infrastructure"
}

object Flavors {
    const val OSS = "oss"
    const val PLAY = "play"

    const val PROD = "prod"
    const val DEVMOLE = "devmole"
    const val STAGEMOLE = "stagemole"
}

data class Variant(val buildType: String?, val productFlavors: Map<String, String>) {
    constructor(
        buildType: String?,
        productFlavors: List<Pair<String, String>>,
    ) : this(buildType, productFlavors.toMap())
}

data class VariantFilter(
    val billingPredicate: (billing: String?) -> Boolean = { true },
    val infrastructurePredicate: (infrastructure: String?) -> Boolean = { true },
    val buildTypePredicate: (buildType: String?) -> Boolean = { true },
)

fun Variant.matches(filter: VariantFilter): Boolean =
    with(filter) {
        val flavors = productFlavors.toMap()
        buildTypePredicate(buildType) &&
            infrastructurePredicate(flavors[FlavorDimensions.INFRASTRUCTURE]) &&
            billingPredicate(flavors[FlavorDimensions.BILLING])
    }

fun Variant.matchesAny(vararg filters: VariantFilter): Boolean = filters.any { matches(it) }

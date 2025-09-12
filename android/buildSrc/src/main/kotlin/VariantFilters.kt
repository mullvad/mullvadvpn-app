import BuildTypes.BENCHMARK
import BuildTypes.DEBUG
import BuildTypes.FDROID
import BuildTypes.LEAK_CANARY
import BuildTypes.NON_MINIFIED
import BuildTypes.RELEASE
import Flavors.OSS
import Flavors.PLAY
import Flavors.PROD
import Flavors.STAGEMOLE

val ossProdAnyBuildType =
    VariantFilter(
        billingPredicate = { it == OSS },
        infrastructurePredicate = { it == PROD },
        buildTypePredicate = {
            when (it) {
                DEBUG,
                RELEASE,
                FDROID,
                LEAK_CANARY -> true
                else -> false
            }
        },
    )

val allPlayDebugReleaseVariants =
    VariantFilter(
        billingPredicate = { it == PLAY },
        buildTypePredicate = { buildType: String? ->
            when (buildType) {
                DEBUG,
                RELEASE -> true
                else -> false
            }
        },
    )

val baselineFilter =
    VariantFilter(
        billingPredicate = { it == PLAY },
        infrastructurePredicate = { it == PROD },
        buildTypePredicate = {
            if (it == null) return@VariantFilter false

            val isBaselineBuildType =
                it.contains(NON_MINIFIED, true) || it.contains(BENCHMARK, true)

            isBaselineBuildType && it.contains(RELEASE, true)
        },
    )

val ossProdDebug =
    VariantFilter(
        billingPredicate = { it == OSS },
        infrastructurePredicate = { it == PROD },
        buildTypePredicate = { it == DEBUG },
    )
val playStagemoleDebug =
    VariantFilter(
        billingPredicate = { it == PLAY },
        infrastructurePredicate = { it == STAGEMOLE },
        buildTypePredicate = { it == DEBUG },
    )

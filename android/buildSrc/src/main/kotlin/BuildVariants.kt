import BuildTypes.DEBUG
import BuildTypes.FDROID
import BuildTypes.LEAK_CANARY
import BuildTypes.RELEASE
import Flavors.DEVMOLE
import Flavors.OSS
import Flavors.PLAY
import Flavors.PROD
import Flavors.STAGEMOLE

object BuildTypes {
    const val DEBUG = "debug"
    const val RELEASE = "release"
    const val FDROID = "fdroid"
    const val LEAK_CANARY = "leakCanary"
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

val enabledAppVariantTriples =
    listOf(
        Triple(OSS, PROD, DEBUG),
        Triple(OSS, PROD, RELEASE),
        Triple(OSS, PROD, FDROID),
        Triple(OSS, PROD, LEAK_CANARY),
        Triple(PLAY, PROD, DEBUG),
        Triple(PLAY, PROD, RELEASE),
        Triple(PLAY, DEVMOLE, DEBUG),
        Triple(PLAY, DEVMOLE, RELEASE),
        Triple(PLAY, STAGEMOLE, DEBUG),
        Triple(PLAY, STAGEMOLE, RELEASE)
    )

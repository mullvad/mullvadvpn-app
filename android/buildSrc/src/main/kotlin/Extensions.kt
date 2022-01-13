fun String.isNonStableVersion(): Boolean {
    val nonStableQualifiers = listOf("alpha", "beta", "rc")

    val isNonStable = nonStableQualifiers
        .map { qualifier -> Regex("(?i).*[.-]$qualifier[.\\d-+]*") }
        .any { it.matches(this) }

    return isNonStable
}

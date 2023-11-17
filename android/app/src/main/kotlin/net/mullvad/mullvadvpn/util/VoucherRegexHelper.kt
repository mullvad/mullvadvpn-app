package net.mullvad.mullvadvpn.util

private const val VALID_VOUCHER_CHARACTER_REGEX_PATTERN = "^[A-Za-z0-9- \r\n]*$"
private const val IGNORED_VOUCHER_CHARACTER_REGEX_PATTERN = """[- \n\r]"""

object VoucherRegexHelper {
    fun validate(input: String): Boolean {
        return VALID_VOUCHER_CHARACTER_REGEX_PATTERN.toRegex().matches(input)
    }

    fun trim(input: String): String =
        input.replace(Regex(IGNORED_VOUCHER_CHARACTER_REGEX_PATTERN), "")
}

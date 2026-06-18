package net.mullvad.mullvadvpn.lib.payment.model

enum class PaymentStatus {
    // The purchase will be paid and verified later.
    PENDING,
    // The purchase has been paid to Google, but not verified towards our API. This could indicate
    // and issue with the verification.
    PURCHASED_UNVERIFIED,
}

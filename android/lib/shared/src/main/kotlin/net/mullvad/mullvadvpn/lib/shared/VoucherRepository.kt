package net.mullvad.mullvadvpn.lib.shared

import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService

class VoucherRepository(
    private val managementService: ManagementService,
    private val accountRepository: AccountRepository
) {
    suspend fun submitVoucher(voucher: String) =
        managementService.submitVoucher(voucher).onRight {
            accountRepository.onVoucherRedeemed(it.newExpiryDate)
        }
}

package net.mullvad.mullvadvpn.lib.repository

import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.VoucherCode

class VoucherRepository(
    private val managementService: ManagementService,
    private val accountRepository: AccountRepository,
) {
    suspend fun submitVoucher(voucher: VoucherCode) =
        managementService.submitVoucher(voucher).onRight {
            accountRepository.onVoucherRedeemed(it.newExpiryDate)
        }
}

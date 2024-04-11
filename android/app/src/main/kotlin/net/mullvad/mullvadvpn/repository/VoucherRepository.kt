package net.mullvad.mullvadvpn.repository

import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService

class VoucherRepository(
    private val managementService: ManagementService,
) {
    suspend fun submitVoucher(voucher: String) = managementService.submitVoucher(voucher)
}

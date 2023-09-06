//
//  AccountVoucherInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-30.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

final class AccountVoucherInteractor: RedeemVoucherProtocol {
    var tasks: [MullvadTypes.Cancellable] = []
    private let tunnelManager: TunnelManager

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
    }

    func redeemVoucher(
        code: String,
        completion: @escaping ((Result<MullvadREST.REST.SubmitVoucherResponse, Error>) -> Void)
    ) {
        tasks.append(tunnelManager.redeemVoucher(code, completion: completion))
    }
}

//
//  RedeemVoucherInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-05-24.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

final class RedeemVoucherInteractor {
    private let tunnelManager: TunnelManager

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
    }

    func redeemVoucher(
        code: String,
        completion: @escaping ((Result<REST.SubmitVoucherResponse, Error>) -> Void)) -> Cancellable {
        tunnelManager.redeemVoucher(code, completion: completion)
    }
}

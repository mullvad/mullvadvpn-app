//
//  RedeemVoucherInteractorProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-30.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

protocol RedeemVoucherProtocol {
    var tasks: [Cancellable] { get }
    func redeemVoucher(
        code: String,
        completion: @escaping ((Result<REST.SubmitVoucherResponse, Error>) -> Void)
    )
    func cancelAll()
}

extension RedeemVoucherProtocol {
    func cancelAll() {
        tasks.forEach { $0.cancel() }
    }
}

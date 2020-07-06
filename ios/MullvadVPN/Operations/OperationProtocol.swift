//
//  OperationProtocol.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol OperationProtocol: Operation {
    /// Add operation observer
    func addObserver<T: OperationObserver>(_ observer: T) where T.OperationType == Self

    /// Finish operation
    func finish()

    /// Cancel operation
    func cancel()
}

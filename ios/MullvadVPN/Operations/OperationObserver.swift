//
//  OperationObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol OperationObserver {
    associatedtype OperationType: OperationProtocol

    func operationWillExecute(_ operation: OperationType)
    func operationWillFinish(_ operation: OperationType)
    func operationDidFinish(_ operation: OperationType)
}


//
//  AnyOperationObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

class AnyOperationObserver<OperationType: OperationProtocol>: OperationBlockObserver<OperationType> {
    init<T: OperationObserver>(_ observer: T) where T.OperationType == OperationType {
        super.init(
            willExecute: observer.operationWillExecute,
            willFinish: observer.operationWillFinish,
            didFinish: observer.operationDidFinish
        )
    }
}

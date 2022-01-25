//
//  TunnelOperationObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 25/01/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

final class TunnelOperationObserver<Success, Failure: Error> {
    private let queue: DispatchQueue?
    private let finishHandler: (Operation, OperationCompletion<Success, Failure>) -> Void

    init(queue: DispatchQueue?, finishHandler: @escaping (Operation, OperationCompletion<Success, Failure>) -> Void) {
        self.queue = queue
        self.finishHandler = finishHandler
    }

    func operationDidFinish(_ operation: Operation, completion: OperationCompletion<Success, Failure>) {
        let handler = {
            self.finishHandler(operation, completion)
        }

        queue?.async(execute: handler) ?? handler()
    }
}

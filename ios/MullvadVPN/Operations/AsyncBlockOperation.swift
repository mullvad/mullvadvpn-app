//
//  AsyncBlockOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 06/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Asynchronous block operation
class AsyncBlockOperation: AsyncOperation {
    private let block: ((AsyncBlockOperation) -> Void)

    init(block: @escaping (AsyncBlockOperation) -> Void) {
        self.block = block
    }

    override func main() {
        block(self)
    }
}

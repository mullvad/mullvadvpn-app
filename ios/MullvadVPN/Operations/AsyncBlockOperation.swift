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
    private let block: ((@escaping () -> Void) -> Void)

    init(_ block: @escaping (@escaping () -> Void) -> Void) {
        self.block = block
        super.init()
    }

    override func main() {
        block { [weak self] in
            self?.finish()
        }
    }
}

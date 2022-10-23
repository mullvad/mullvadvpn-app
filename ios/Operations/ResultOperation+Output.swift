//
//  ResultOperation+Output.swift
//  Operations
//
//  Created by pronebird on 31/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension ResultOperation: OutputOperation {
    public var output: Success? {
        return completion?.value
    }
}

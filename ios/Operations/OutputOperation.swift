//
//  OutputOperation.swift
//  Operations
//
//  Created by pronebird on 31/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol OutputOperation: Operation {
    associatedtype Output

    var output: Output? { get }
}

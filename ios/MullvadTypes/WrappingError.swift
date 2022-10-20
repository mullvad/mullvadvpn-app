//
//  WrappingError.swift
//  MullvadTypes
//
//  Created by pronebird on 23/09/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Protocol describing errors that may contain underlying errors.
public protocol WrappingError: Error {
    var underlyingError: Error? { get }
}

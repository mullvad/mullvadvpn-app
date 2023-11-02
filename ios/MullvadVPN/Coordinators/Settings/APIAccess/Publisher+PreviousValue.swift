//
//  Publisher+PreviousValue.swift
//  MullvadVPN
//
//  Created by pronebird on 27/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation

extension Publisher {
    /// A publisher producing a pair that contains the previous and new value.
    ///
    /// - Returns: A publisher emitting a tuple containing the previous and new value.
    func withPreviousValue() -> some Publisher<(Output?, Output), Failure> {
        return scan(nil) { ($0?.1, $1) }.compactMap { $0 }
    }
}

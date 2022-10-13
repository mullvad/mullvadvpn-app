//
//  Cancellable.swift
//  MullvadVPN
//
//  Created by pronebird on 15/03/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol Cancellable {
    func cancel()
}

extension Operation: Cancellable {}

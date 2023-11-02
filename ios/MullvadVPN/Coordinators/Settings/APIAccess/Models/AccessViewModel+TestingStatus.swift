//
//  AccessViewModel+TestingStatus.swift
//  MullvadVPN
//
//  Created by pronebird on 27/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension AccessMethodViewModel.TestingStatus {
    var sheetStatus: AccessMethodActionSheetContentConfiguration.Status {
        switch self {
        case .initial:
            // The sheet is invisible in this state, the return value is not important.
            .testing
        case .inProgress:
            .testing
        case .failed:
            .unreachable
        case .succeeded:
            .reachable
        }
    }
}

//
//  ProcessEnvironment+Extension.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-12-18.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

#if DEBUG
/// Available overrides that can be passed on launch only in `Debug` mode.
private enum ArgumentKey {
    static let changeLog = "SHOW_CHANGELOG"
}

extension ProcessInfo {
    /// Forces app to show change log view controller.
    static var shouldShowChangeLog: Bool {
        return processInfo.arguments.contains(ArgumentKey.changeLog)
    }
}
#endif

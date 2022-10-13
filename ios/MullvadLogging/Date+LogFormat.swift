//
//  Date+LogFormat.swift
//  LogFormatting
//
//  Created by pronebird on 09/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Date {
    public func logFormatDate() -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "dd/MM/yyyy @ HH:mm:ss"

        return formatter.string(from: self)
    }
}

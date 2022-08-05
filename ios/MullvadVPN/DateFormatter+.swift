//
//  DateFormatter+.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-25.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension DateFormatter {
    enum DateFormat: String {
        case iso8601 = "yyyy-MM-dd'T'HH:mm:ss+00:00"
        case standard = "dd MMM yyyy, HH:mm"
    }

    convenience init(dateFormat: DateFormat) {
        self.init()
        self.dateFormat = dateFormat.rawValue
    }
}

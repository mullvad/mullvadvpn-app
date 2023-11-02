//
//  AccessMethodHeaderFooterReuseIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Header footer view reuse identifier used in view controllers implementing access method management.
enum AccessMethodHeaderFooterReuseIdentifier: String, CaseIterable, HeaderFooterIdentifierProtocol {
    case primary

    var headerFooterClass: AnyClass {
        switch self {
        case .primary: UITableViewHeaderFooterView.self
        }
    }
}

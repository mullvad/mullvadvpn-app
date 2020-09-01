//
//  SegueIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 25/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

// A phantom struct holding the storyboard segue identifiers for each view controller
enum SegueIdentifier {}

extension SegueIdentifier {

    enum Login: String, SegueConvertible {
        case showConnect = "ShowConnect"
    }

    enum Settings: String, SegueConvertible {
        case showAccount = "ShowAccount"
    }

    enum Account: String, SegueConvertible {
        case logout = "Logout"
    }
}

protocol SegueConvertible: RawRepresentable {
    static func from(segue: UIStoryboardSegue) -> Self?
}

extension SegueConvertible where RawValue == String {
    static func from(segue: UIStoryboardSegue) -> Self? {
        if let identifier = segue.identifier {
            return self.init(rawValue: identifier)
        } else {
            return nil
        }
    }
}

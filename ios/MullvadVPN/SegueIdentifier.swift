//
//  SegueIdentifier.swift
//  MullvadVPN
//
//  Created by pronebird on 25/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

// A phantom struct holding the storyboard segue identifiers for each view controller
struct SegueIdentifier {

    enum Root: String, SegueConvertible {
        case showSettings = "ShowSettings"
    }

    enum Login: String, SegueConvertible {
        case showConnect = "ShowConnect"
    }

    enum SelectLocation: String, SegueConvertible {
        case returnToConnectWithNewRelay = "ReturnToConnectWithNewRelay"
    }

    enum Account: String, SegueConvertible {
        case logout = "Logout"
    }

    private init() {}
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

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

    enum Connect: String, SegueConvertible {
        case embedHeader = "EmbedHeaderBar"
        case showSettings = "ShowSettings"
    }

    enum Login: String, SegueConvertible {
        case embedHeader = "EmbedHeaderBar"
        case showSettings = "ShowSettings"
        case showConnect = "ShowConnect"
    }

    enum SelectLocation: String, SegueConvertible {
        case returnToConnectWithNewRelay = "ReturnToConnectWithNewRelay"
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

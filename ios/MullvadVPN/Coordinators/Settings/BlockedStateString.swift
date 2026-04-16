//
//  BlockedStateString.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-05-08.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

enum BlockedStateString {
    enum Message: CustomStringConvertible {
        case daita
        case multihop
        case obfuscation

        var description: String {
            let customString =
                switch self {
                case .daita:
                    NSLocalizedString("enabling DAITA", comment: "")
                case .multihop:
                    NSLocalizedString("changing mode", comment: "Multihop mode")
                case .obfuscation:
                    NSLocalizedString("selecting this method", comment: "Obfuscation method")
                }

            return [
                String(
                    format: NSLocalizedString(
                        "By %@ your connection will be blocked because the location selected is not compatible with your settings.",
                        comment:
                            "Variable refers to enabling DAITA, changing multihop mode or selecting an obfuscation method"
                    ),
                    customString
                ),
                String(
                    format: NSLocalizedString(
                        "Please select a compatible location after %@.",
                        comment:
                            "Variable refers to enabling DAITA, changing multihop mode or selecting an obfuscation method"
                    ),
                    customString
                ),
            ].joinedParagraphs(lineBreaks: 1)
        }
    }

    enum Button: CustomStringConvertible {
        case daita
        case multihop(MultihopState)
        case obfuscation(WireGuardObfuscationState)

        var description: String {
            switch self {
            case .daita:
                NSLocalizedString("Enable DAITA", comment: "")
            case .multihop(let state):
                String(
                    format: NSLocalizedString(
                        "Change to “%@“",
                        comment: "Variable refers to multihop mode"
                    ),
                    state.description
                )
            case .obfuscation(let state):
                String(
                    format: NSLocalizedString(
                        "Select %@",
                        comment: "Variable refers to obfuscation method"
                    ),
                    state.description
                )
            }
        }
    }
}

//
//  NSParagraphStyle+Extensions.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-06-27.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension NSParagraphStyle {
    static var alert: NSParagraphStyle {
        let style = NSMutableParagraphStyle()

        style.paragraphSpacing = 16
        style.lineBreakMode = .byWordWrapping

        return style
    }
}

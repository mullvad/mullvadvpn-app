//
//  ChangeLogViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-22.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit.NSAttributedString
import UIKit.UIFont

struct ChangeLogViewModel {
    let header: String = Bundle.main.shortVersion
    let title: String = NSLocalizedString(
        "CHANGE_LOG_TITLE",
        tableName: "Account",
        value: "Changes in this version:",
        comment: ""
    )
    let body: NSAttributedString

    init(body: [String]) {
        self.body = body.changeLogAttributedString
    }
}

fileprivate extension Array where Element == String {
    var changeLogAttributedString: NSAttributedString {
        let bullet = "•  "
        let font = UIFont.preferredFont(forTextStyle: .body)
        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.lineBreakMode = .byWordWrapping
        paragraphStyle.headIndent = bullet.size(withAttributes: [.font: font]).width

        return NSAttributedString(
            string: self.map {
                "\(bullet)\($0)"
            }
            .joined(separator: "\n"),
            attributes: [
                .paragraphStyle: paragraphStyle,
                .font: font,
                .foregroundColor: UIColor.white.withAlphaComponent(0.8),
            ]
        )
    }
}

//
//  ChangeLogCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 24/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import UIKit

final class ChangeLogCoordinator: Coordinator, Presentable {
    private let logger = Logger(label: "ChangeLogCoordinator")

    private var alertController: CustomAlertViewController?

    var presentedViewController: UIViewController {
        return alertController!
    }

    var didFinish: (() -> Void)?

    func start() {
        alertController = CustomAlertViewController(
            header: Bundle.main.shortVersion,
            title: NSLocalizedString(
                "CHANGE_LOG_TITLE",
                tableName: "Account",
                value: "Changes in this version:",
                comment: ""
            ),
            attributedMessage: readChangeLogFromFile()
        )

        alertController?.addAction(
            title: NSLocalizedString(
                "CHANGE_LOG_OK_ACTION",
                tableName: "Account",
                value: "Got it!",
                comment: ""
            ),
            style: .default,
            handler: { [weak self] in
                self?.didFinish?()
            }
        )
    }

    private func readChangeLogFromFile() -> NSAttributedString? {
        guard let changeLogText = try? ChangeLog.readFromFile() else {
            logger.error("Cannot read changelog from bundle.")
            return nil
        }

        let bullet = "•  "
        let font = UIFont.preferredFont(forTextStyle: .body)

        let bulletList = changeLogText.split(whereSeparator: { $0.isNewline })
            .map { "\(bullet)\($0)" }
            .joined(separator: "\n")

        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.lineBreakMode = .byWordWrapping
        paragraphStyle.headIndent = bullet.size(withAttributes: [.font: font]).width

        return NSAttributedString(
            string: bulletList,
            attributes: [
                .paragraphStyle: paragraphStyle,
                .font: font,
                .foregroundColor: UIColor.white.withAlphaComponent(0.8),
            ]
        )
    }
}

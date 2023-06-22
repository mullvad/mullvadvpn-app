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
    private let navigationController: RootContainerViewController

    var presentedViewController: UIViewController {
        return navigationController
    }

    init(navigationController: RootContainerViewController) {
        self.navigationController = navigationController
    }

    func start(animated: Bool) {
        ChangeLog.markAsSeen()

        guard let changeLogText = getChangeLogText() else {
            return
        }

        let alertController = CustomAlertViewController(
            title: NSLocalizedString(
                "CHANGE_LOG_TITLE",
                tableName: "Account",
                value: "Changes in this version:",
                comment: ""
            ),
            attributedMessage: changeLogText
        )

        alertController.addVersionHeader()

        alertController.addAction(
            title: NSLocalizedString(
                "CHANGE_LOG_OK_ACTION",
                tableName: "Account",
                value: "Got it!",
                comment: ""
            ),
            style: .default
        )

        presentedViewController.present(alertController, animated: animated)
    }

    private func getChangeLogText() -> NSAttributedString? {
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

private extension CustomAlertViewController {
    func addVersionHeader() {
        let header = UILabel()

        header.text = Bundle.main.shortVersion
        header.font = .preferredFont(forTextStyle: .largeTitle, weight: .bold)
        header.textColor = .white
        header.adjustsFontForContentSizeCategory = true
        header.textAlignment = .center
        header.numberOfLines = 0

        if let title = containerView.arrangedSubviews.first {
            containerView.setCustomSpacing(8, after: title)
        }

        containerView.insertArrangedSubview(header, at: 0)
        containerView.setCustomSpacing(16, after: header)
    }
}

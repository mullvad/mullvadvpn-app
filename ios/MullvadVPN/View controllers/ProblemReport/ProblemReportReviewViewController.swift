//
//  ProblemReportReviewViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 10/02/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class ProblemReportReviewViewController: UIViewController {
    private var textView = UITextView()
    private let interactor: ProblemReportInteractor

    init(interactor: ProblemReportInteractor) {
        self.interactor = interactor
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.accessibilityIdentifier = .appLogsView

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "ProblemReportReview",
            value: "App logs",
            comment: ""
        )

        navigationItem.rightBarButtonItem = UIBarButtonItem(
            systemItem: .done,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.dismiss(animated: true)
            })
        )
        navigationItem.rightBarButtonItem?.accessibilityIdentifier = .appLogsDoneButton

        #if DEBUG
        navigationItem.leftBarButtonItem = UIBarButtonItem(
            systemItem: .action,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.share()
            })
        )
        navigationItem.leftBarButtonItem?.accessibilityIdentifier = .appLogsShareButton
        #endif

        textView.accessibilityIdentifier = .problemReportAppLogsTextView
        textView.translatesAutoresizingMaskIntoConstraints = false
        textView.isEditable = false
        textView.font = UIFont.monospacedSystemFont(
            ofSize: UIFont.systemFontSize,
            weight: .regular
        )
        textView.backgroundColor = .systemBackground

        view.addSubview(textView)

        NSLayoutConstraint.activate([
            textView.topAnchor.constraint(equalTo: view.topAnchor),
            textView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            textView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            textView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])

        // Used to layout constraints so that navigation controller could properly adjust the text
        // view insets.
        view.layoutIfNeeded()

        loadLogs()
    }

    override func selectAll(_ sender: Any?) {
        textView.selectAll(sender)
    }

    private func loadLogs() {
        let presentation = AlertPresentation(
            id: "problem-report-load",
            icon: .spinner,
            buttons: []
        )

        let alertController = AlertViewController(presentation: presentation)

        present(alertController, animated: true) {
            self.textView.text = self.interactor.reportString
            self.dismiss(animated: true)
        }
    }

    #if DEBUG
    private func share() {
        let activityController = UIActivityViewController(
            activityItems: [interactor.reportString],
            applicationActivities: nil
        )

        activityController.popoverPresentationController?.barButtonItem = navigationItem.leftBarButtonItem

        present(activityController, animated: true)
    }
    #endif
}

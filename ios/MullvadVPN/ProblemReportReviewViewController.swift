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
    private let reportString: String

    private var dismissButtonItem: UIBarButtonItem {
        return UIBarButtonItem(
            barButtonSystemItem: .done,
            target: self,
            action: #selector(handleDismissButton(_:))
        )
    }

    init(reportString: String) {
        self.reportString = reportString
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "ProblemReportReview",
            value: "App logs",
            comment: ""
        )
        navigationItem.rightBarButtonItem = dismissButtonItem

        textView.translatesAutoresizingMaskIntoConstraints = false
        textView.text = reportString
        textView.isEditable = false
        textView.font = UIFont.backport_monospacedSystemFont(
            ofSize: UIFont.systemFontSize,
            weight: .regular
        )

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
    }

    override func selectAll(_ sender: Any?) {
        textView.selectAll(sender)
    }

    // MARK: - Actions

    @objc func handleDismissButton(_ sender: Any) {
        dismiss(animated: true)
    }
}

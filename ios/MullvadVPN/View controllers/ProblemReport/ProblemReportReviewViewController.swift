//
//  ProblemReportReviewViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 10/02/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class ProblemReportReviewViewController: UIViewController {
    private let spinnerView = SpinnerActivityIndicatorView(style: .large)
    private var textView = UITextView()
    private let interactor: ProblemReportInteractor
    private lazy var spinnerContainerView: UIView = {
        let view = UIView()
        view.backgroundColor = .black.withAlphaComponent(0.5)
        return view
    }()

    init(interactor: ProblemReportInteractor) {
        self.interactor = interactor
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .secondaryColor
        view.setAccessibilityIdentifier(.appLogsView)

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
        navigationItem.rightBarButtonItem?.setAccessibilityIdentifier(.appLogsDoneButton)

        #if DEBUG
        navigationItem.leftBarButtonItem = UIBarButtonItem(
            systemItem: .action,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.share()
            })
        )
        navigationItem.leftBarButtonItem?.setAccessibilityIdentifier(.appLogsShareButton)
        #endif

        textView.setAccessibilityIdentifier(.problemReportAppLogsTextView)
        textView.translatesAutoresizingMaskIntoConstraints = false
        textView.isEditable = false
        textView.font = .mullvadSmall
        textView.adjustsFontForContentSizeCategory = true
        textView.backgroundColor = .systemBackground
        textView.textAlignment = .left

        view.addConstrainedSubviews([textView]) {
            textView.pinEdgesToSuperview(.all().excluding(.top))
            textView.pinEdgeToSuperviewMargin(.top(0))
        }

        textView.addConstrainedSubviews([spinnerContainerView]) {
            spinnerContainerView.pinEdgesToSuperview()
            spinnerContainerView.widthAnchor.constraint(equalTo: textView.widthAnchor)
            spinnerContainerView.heightAnchor.constraint(equalTo: textView.heightAnchor)
        }

        spinnerContainerView.addConstrainedSubviews([spinnerView]) {
            spinnerView.centerXAnchor.constraint(equalTo: view.centerXAnchor)
            spinnerView.centerYAnchor.constraint(equalTo: view.centerYAnchor)
        }

        // Used to layout constraints so that navigation controller could properly adjust the text
        // view insets.
        view.layoutIfNeeded()

        loadLogs()
    }

    override func selectAll(_ sender: Any?) {
        textView.selectAll(sender)
    }

    private func loadLogs() {
        spinnerView.startAnimating()
        interactor.fetchReportString { [weak self] reportString in
            guard let self else { return }
            Task { @MainActor in
                textView.text = reportString
                spinnerView.stopAnimating()
                spinnerContainerView.isHidden = true
            }
        }
    }

    #if DEBUG
    private func share() {
        interactor.fetchReportString { [weak self] reportString in
            guard let self,!reportString.isEmpty else { return }
            Task { @MainActor in
                let activityController = UIActivityViewController(
                    activityItems: [reportString],
                    applicationActivities: nil
                )

                activityController.popoverPresentationController?.barButtonItem = navigationItem.leftBarButtonItem

                present(activityController, animated: true)
            }
        }
    }
    #endif
}

//
//  IPOverrideTextViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

class IPOverrideTextViewController: UIViewController {
    private var textView = CustomTextView()

    private lazy var importButton: UIBarButtonItem = {
        return UIBarButtonItem(
            title: NSLocalizedString(
                "IMPORT_TEXT_IMPORT_BUTTON",
                tableName: "IPOverride",
                value: "Import",
                comment: ""
            ),
            primaryAction: UIAction(handler: { [weak self] _ in
                guard let self else { return }

                didFinishEditing?(textView.text)
                dismiss(animated: true)
            })
        )
    }()

    var didFinishEditing: ((String) -> Void)?

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor

        navigationItem.title = NSLocalizedString(
            "IMPORT_TEXT_NAVIGATION_TITLE",
            tableName: "IPOverride",
            value: "Import via text",
            comment: ""
        )

        navigationItem.leftBarButtonItem = UIBarButtonItem(
            systemItem: .cancel,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.dismiss(animated: true)
            })
        )

        importButton.isEnabled = !textView.text.isEmpty
        navigationItem.rightBarButtonItem = importButton

        textView.becomeFirstResponder()
        textView.delegate = self
        textView.spellCheckingType = .no
        textView.autocorrectionType = .no
        textView.font = UIFont.monospacedSystemFont(
            ofSize: UIFont.systemFont(ofSize: 14).pointSize,
            weight: .regular
        )

        view.addConstrainedSubviews([textView]) {
            textView.pinEdgesToSuperview(.all().excluding(.top))
            textView.topAnchor.constraint(equalTo: view.layoutMarginsGuide.topAnchor, constant: 0)
        }
    }
}

extension IPOverrideTextViewController: UITextViewDelegate {
    func textViewDidChange(_ textView: UITextView) {
        importButton.isEnabled = !textView.text.isEmpty
    }
}

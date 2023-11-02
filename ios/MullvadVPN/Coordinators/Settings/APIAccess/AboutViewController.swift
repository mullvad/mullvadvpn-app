//
//  AboutViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 08/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// View controller used for presenting a detailed information on some topic using markdown in a scrollable text view.
class AboutViewController: UIViewController {
    private let textView = UITextView()
    private let markdown: String

    init(markdown: String) {
        self.markdown = markdown
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.paragraphSpacing = 16

        let stylingOptions = MarkdownStylingOptions(
            font: .systemFont(ofSize: 17),
            paragraphStyle: paragraphStyle
        )

        textView.attributedText = NSAttributedString(markdownString: markdown, options: stylingOptions)
        textView.textContainerInset = UIMetrics.contentInsets
        textView.isEditable = false

        view.addConstrainedSubviews([textView]) {
            textView.pinEdgesToSuperview()
        }
    }
}

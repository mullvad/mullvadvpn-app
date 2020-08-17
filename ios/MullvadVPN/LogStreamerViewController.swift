//
//  LogStreamerViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 17/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

#if DEBUG

import Foundation
import UIKit

class LogStreamerViewController: UIViewController, UITextViewDelegate {

    private let textView = UITextView()
    private let streamer: LogStreamer<UTF8>

    private var autoScrollButtonItem: UIBarButtonItem {
        return UIBarButtonItem(barButtonSystemItem: autoScroll ? .pause : .play, target: self, action: #selector(handleToggleAutoscroll(_:)))
    }

    private var dismissButtonItem: UIBarButtonItem {
        return UIBarButtonItem(barButtonSystemItem: .done, target: self, action: #selector(handleDismissButton(_:)))
    }

    var autoScroll: Bool = true {
        didSet {
            updateAutoScrollBarItem()
            handleAutoScroll()
        }
    }

    init(fileURLs: [URL]) {
        streamer = LogStreamer(fileURLs: fileURLs)
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        navigationItem.title = NSLocalizedString("App logs", comment: "")
        navigationItem.leftBarButtonItem = autoScrollButtonItem
        navigationItem.rightBarButtonItem = dismissButtonItem

        addSubviews()
        startStreamer()
    }

    // MARK: - UITextViewDelegate

    func scrollViewWillEndDragging(_ scrollView: UIScrollView, withVelocity velocity: CGPoint, targetContentOffset: UnsafeMutablePointer<CGPoint>) {
        let translation = scrollView.panGestureRecognizer.translation(in: scrollView.superview)

        // Disable autoscroll if user scrolled up
        if translation.y > 0 {
            autoScroll = false
        }
    }

    func scrollViewShouldScrollToTop(_ scrollView: UIScrollView) -> Bool {
        // Disable autoscroll when user requested scroll to top
        autoScroll = false
        return true
    }

    // MARK: - Private

    private func addSubviews() {
        textView.translatesAutoresizingMaskIntoConstraints = false
        textView.isEditable = false
        textView.delegate = self

        view.addSubview(textView)

        NSLayoutConstraint.activate([
            textView.topAnchor.constraint(equalTo: view.topAnchor),
            textView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            textView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            textView.bottomAnchor.constraint(equalTo: view.bottomAnchor)
        ])
    }

    private func startStreamer() {
        self.streamer.start { [weak self] (str) in
            guard let self = self else { return }

            DispatchQueue.main.async {
                self.textView.insertText("\(str)\n")
                self.handleAutoScroll()
            }
        }
    }

    private func handleAutoScroll() {
        if autoScroll && !textView.isTracking && !textView.isDragging {
            scrollToBottom()
        }
    }

    private func scrollToBottom() {
        let textRange = NSRange(..<textView.text.endIndex, in: textView.text)

        textView.scrollRangeToVisible(textRange)
    }

    private func updateAutoScrollBarItem() {
        navigationItem.leftBarButtonItem = autoScrollButtonItem
    }

    // MARK: - Actions

    @objc func handleDismissButton(_ sender: Any) {
        dismiss(animated: true)
    }

    @objc func handleToggleAutoscroll(_ sender: Any) {
        autoScroll = !autoScroll
    }
}

#endif

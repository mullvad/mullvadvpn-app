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
import Logging

class LogStreamerViewController: UIViewController, UITextViewDelegate {

    private let textView = UITextView()
    private let streamer: LogStreamer<UTF8>
    private let logEntryParser = LogEntryParser()
    private var currentTextColor: UIColor?
    private let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss.SSS"
        return formatter
    }()

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

        navigationItem.title = "App logs"
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
        } else if translation.y < 0 {
            // Enable autoscroll if user scrolled to the bottom of the view
            let maxScrollY = scrollView.contentSize.height - scrollView.frame.height

            if targetContentOffset.pointee.y >= maxScrollY {
                autoScroll = true
            }
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
        if #available(iOS 13.0, *) {
            textView.font = UIFont.monospacedSystemFont(ofSize: UIFont.systemFontSize, weight: .regular)
        } else {
            textView.font = UIFont(name: "Courier", size: UIFont.systemFontSize)
        }
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
                // Try parsing the entry
                let entry = self.logEntryParser.parse(str)

                // Since the log streamer sends the log file line-by-line, it's possible that only a
                // part of a multiline message is captured at first.
                let message = entry.map { (entry) -> String in
                    // Reformat the log entry date
                    let timestamp = self.timestampFormatter.string(from: entry.timestamp)

                    return "\(timestamp) \(entry.module) \(entry.message)\n"
                } ?? "\(str)\n"


                // Compute the range for replacing the text color
                let start = self.textView.text.utf16.count
                let end = start + message.utf16.count
                let textRange = NSRange(start..<end)

                self.textView.insertText(message)
                self.handleAutoScroll()

                // Update the current log entry color
                if let logLevel = entry?.level {
                    self.currentTextColor = self.textColor(for: logLevel)
                }

                // Apply the color attribute to the inserted text
                if let textColor = self.currentTextColor {
                    self.textView.textStorage.addAttributes([.foregroundColor: textColor], range: textRange)
                }
            }
        }
    }

    private func handleAutoScroll() {
        if autoScroll && !textView.isTracking && (!textView.isDragging || textView.isDecelerating) {
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

    private func textColor(for logLevel: Logger.Level) -> UIColor {
        switch logLevel {
        case .debug, .trace:
            return .lightGray
        case .error, .critical:
            return .red
        case .info, .notice:
            return .blue
        case .warning:
            return .orange
        }
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

//
//  ScaledSegmentedControl.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-07-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import UIKit

final class ScaledSegmentedControl: UISegmentedControl {
    private let textStyle: UIFont.TextStyle
    private let fontWeight: UIFont.Weight

    init(textStyle: UIFont.TextStyle = .body, weight: UIFont.Weight = .regular) {
        self.textStyle = textStyle
        self.fontWeight = weight
        super.init(frame: .zero)
        applyTextAttributes()
        subscribeToDynamicType()
    }

    required init?(coder: NSCoder) {
        self.textStyle = .body
        self.fontWeight = .regular
        super.init(coder: coder)
        applyTextAttributes()
        subscribeToDynamicType()
    }

    override func insertSegment(withTitle title: String?, at segment: Int, animated: Bool) {
        super.insertSegment(withTitle: title, at: segment, animated: animated)
        applyTextAttributes()
    }

    private func applyTextAttributes() {
        let font = UIFont.preferredFont(forTextStyle: textStyle).withWeight(fontWeight)
        let attributes: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: UIColor.white,
        ]
        setTitleTextAttributes(attributes, for: .normal)
        setTitleTextAttributes(attributes, for: .selected)
    }

    private func subscribeToDynamicType() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(contentSizeChanged),
            name: UIContentSizeCategory.didChangeNotification,
            object: nil
        )
    }

    @objc private func contentSizeChanged() {
        applyTextAttributes()
    }

    deinit {
        NotificationCenter.default.removeObserver(self)
    }
}

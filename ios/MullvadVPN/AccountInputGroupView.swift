//
//  AccountInputGroupView.swift
//  MullvadVPN
//
//  Created by pronebird on 22/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

@IBDesignable class AccountInputGroupView: UIView {

    @IBOutlet var textField: UITextField!

    private let borderRadius = CGFloat(8)
    private let borderWidth = CGFloat(2)

    private let borderLayer = CAShapeLayer()
    private let backgroundLayer = CAShapeLayer()
    private let maskLayer = CALayer()

    override init(frame: CGRect) {
        super.init(frame: frame)
        setup()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
        setup()
    }

    // MARK: - CALayerDelegate

    override func layoutSublayers(of layer: CALayer) {
        super.layoutSublayers(of: layer)

        guard layer == self.layer else { return }

        // extend the border frame outside of the content area
        let borderFrame = layer.bounds.insetBy(dx: -borderWidth * 0.5, dy: -borderWidth * 0.5)

        // create a bezier path for border
        let borderPath = borderBezierPath(size: borderFrame.size)

        // update the background layer mask
        maskLayer.frame.size = borderFrame.size
        maskLayer.contents = backgroundMaskImage(borderPath: borderPath).cgImage

        backgroundLayer.frame = borderFrame

        borderLayer.path = borderPath.cgPath
        borderLayer.frame = borderFrame
    }

    // MARK: - Notifications

    @objc func textDidBeginEditing() {
        updateBorderStyle()
    }

    @objc func textDidEndEditing() {
        updateBorderStyle()
    }

    // MARK: - Private

    private func setup() {
        backgroundColor = UIColor.clear

        borderLayer.lineWidth = borderWidth
        borderLayer.strokeColor = UIColor.clear.cgColor
        borderLayer.fillColor = UIColor.clear.cgColor

        backgroundLayer.backgroundColor = UIColor.white.cgColor
        backgroundLayer.mask = maskLayer

        layer.insertSublayer(borderLayer, at: 0)
        layer.insertSublayer(backgroundLayer, at: 0)

        addTextFieldNotificationObservers()
    }


    private func addTextFieldNotificationObservers() {
        NotificationCenter.default.addObserver(self, selector: #selector(textDidBeginEditing), name: UITextField.textDidBeginEditingNotification, object: textField)
        NotificationCenter.default.addObserver(self, selector: #selector(textDidEndEditing), name: UITextField.textDidEndEditingNotification, object: textField)
    }

    private func updateBorderStyle() {
        let borderColor = textField.isEditing ? UIColor.accountTextFieldBorderColor : UIColor.clear

        borderLayer.strokeColor = borderColor.cgColor
    }

    private func borderBezierPath(size: CGSize) -> UIBezierPath {
        let borderPath = UIBezierPath(roundedRect: CGRect(origin: .zero, size: size), cornerRadius: borderRadius)
        borderPath.lineWidth = borderWidth

        return borderPath
    }

    private func backgroundMaskImage(borderPath: UIBezierPath) -> UIImage {
        let renderer = UIGraphicsImageRenderer(bounds: borderPath.bounds)
        return renderer.image { (ctx) in
            borderPath.fill()

            // strip out any overlapping pixels between the border and the background
            borderPath.stroke(with: .clear, alpha: 0)
        }
    }
}

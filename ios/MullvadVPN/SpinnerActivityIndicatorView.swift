//
//  SpinnerActivityIndicatorView.swift
//  MullvadVPN
//
//  Created by pronebird on 15/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

private let kRotationAnimationKey = "rotation"
private let kAnimationDuration = 0.6

class SpinnerActivityIndicatorView: UIView {

    enum Style {
        case small, medium, large

        var intrinsicSize: CGSize {
            switch self {
            case .small:
                return .init(width: 16, height: 16)
            case .medium:
                return .init(width: 20, height: 20)
            case .large:
                return .init(width: 48, height: 48)
            }
        }

        var thickness: CGFloat {
            switch self {
            case .small, .medium:
                return 2
            case .large:
                return 6
            }
        }
    }

    /// Thickness of the front and back circles
    var thickness: CGFloat = 6 {
        didSet {
            setLayersThickness()
        }
    }

    /// The back circle color
    var backCircleColor = UIColor.white.withAlphaComponent(0.2) {
        didSet {
            setBackCircleLayerColor()
        }
    }

    /// The front circle color
    var frontCircleColor: UIColor? {
        didSet {
            setFrontCircleLayerColor()
        }
    }

    private(set) var isAnimating = false
    private(set) var style = Style.large

    fileprivate let frontCircle = CAShapeLayer()
    fileprivate let backCircle = CAShapeLayer()
    fileprivate var startTime = CFTimeInterval(0)
    fileprivate var stopTime = CFTimeInterval(0)

    override var intrinsicContentSize: CGSize {
        return style.intrinsicSize
    }

    convenience init(style: Style) {
        self.init(frame: .init(origin: .zero, size: style.intrinsicSize))
        self.style = style
        self.thickness = style.thickness
        commonInit()
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        commonInit()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
        commonInit()
    }

    deinit {
        unregisterFromAppStateNotifications()
    }

    override func layoutSublayers(of layer: CALayer) {
        super.layoutSublayers(of: layer)
        setupBezierPaths()
    }

    override func didMoveToWindow() {
        super.didMoveToWindow()

        if window != nil {
            restartAnimationIfNeeded()
        }
    }

    override func tintColorDidChange() {
        super.tintColorDidChange()

        setFrontCircleLayerColor()
    }

    func startAnimating() {
        guard !isAnimating else { return }
        isAnimating = true

        isHidden = false
        addAnimation()
    }

    func stopAnimating() {
        guard isAnimating else { return }
        isAnimating = false

        isHidden = true
        removeAnimation()
    }

    // MARK: - Private

    private func commonInit() {
        registerForAppStateNotifications()

        isHidden = true
        backgroundColor = UIColor.clear

        backCircle.fillColor = UIColor.clear.cgColor
        frontCircle.fillColor = UIColor.clear.cgColor
        frontCircle.lineCap = .round

        setBackCircleLayerColor()
        setFrontCircleLayerColor()
        setLayersThickness()

        layer.addSublayer(backCircle)
        layer.addSublayer(frontCircle)
    }

    private func setLayersThickness() {
        backCircle.lineWidth = thickness
        frontCircle.lineWidth = thickness
    }

    private func setBackCircleLayerColor() {
        backCircle.strokeColor = backCircleColor.cgColor
    }

    private func setFrontCircleLayerColor() {
        frontCircle.strokeColor = frontCircleColor?.cgColor ?? tintColor.cgColor
    }

    private func addAnimation() {
        let timeOffset = stopTime - startTime

        let anim = animation()
        anim.timeOffset = timeOffset

        layer.add(anim, forKey: kRotationAnimationKey)

        startTime = layer.convertTime(CACurrentMediaTime(), from: nil) - timeOffset
    }

    private func removeAnimation() {
        layer.removeAnimation(forKey: kRotationAnimationKey)

        stopTime = layer.convertTime(CACurrentMediaTime(), from: nil)
    }

    @objc private func restartAnimationIfNeeded() {
        let anim = layer.animation(forKey: kRotationAnimationKey)

        if isAnimating && anim == nil {
            removeAnimation()
            addAnimation()
        }
    }

    private func registerForAppStateNotifications() {
        NotificationCenter.default.addObserver(self, selector: #selector(restartAnimationIfNeeded), name: UIApplication.willEnterForegroundNotification, object: nil)
    }

    private func unregisterFromAppStateNotifications() {
        NotificationCenter.default.removeObserver(self)
    }

    private func animation() -> CABasicAnimation {
        let animation = CABasicAnimation(keyPath: "transform.rotation")
        animation.toValue = NSNumber(value: Double.pi * 2)
        animation.duration = kAnimationDuration
        animation.repeatCount = Float.infinity
        animation.timingFunction = CAMediaTimingFunction(name: .linear)

        return animation
    }

    private func setupBezierPaths() {
        let center = CGPoint(x: bounds.size.width * 0.5, y: bounds.size.height * 0.5)
        let radius = bounds.size.width * 0.5 - thickness
        let closedRingPath = UIBezierPath(arcCenter: center, radius: radius, startAngle: 0, endAngle: CGFloat.pi * 2, clockwise: true)
        let openRingPath = UIBezierPath(arcCenter: center, radius: radius, startAngle: 0, endAngle: CGFloat.pi * 1.5, clockwise: true)

        backCircle.path = closedRingPath.cgPath
        frontCircle.path = openRingPath.cgPath
    }

}

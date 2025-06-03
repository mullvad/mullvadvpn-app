import SwiftUI

struct ClearBackgroundView: UIViewRepresentable {
    func makeUIView(context: Context) -> UIView {
        return InnerView()
    }

    func updateUIView(_ uiView: UIView, context: Context) {}

    private class InnerView: UIView {
        override func didMoveToWindow() {
            super.didMoveToWindow()

            superview?.superview?.backgroundColor = .init(red: 0, green: 0, blue: 0, alpha: 0.5)
        }
    }
}

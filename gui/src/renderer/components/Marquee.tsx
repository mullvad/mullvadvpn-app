import * as React from 'react';
import { Animated, Component, Styles, Types, UserInterface, View } from 'reactxp';

const styles = {
  text: Styles.createTextStyle({
    // @ts-ignore
    width: 'fit-content',
    whiteSpace: 'nowrap',
  }),
};

interface IMarqueeProps {
  style?: Types.StyleRuleSetRecursive<Types.ButtonStyleRuleSet>;
}

export default class Marquee extends Component<IMarqueeProps> {
  private initialLeft = Animated.createValue(0.0);
  private textAnimation = Styles.createAnimatedTextStyle({ left: this.initialLeft });
  private textRef = React.createRef<Animated.Text>();

  public componentDidMount() {
    this.startAnimation();
  }

  public componentDidUpdate() {
    this.startAnimation();
  }

  public render() {
    return (
      <View>
        <Animated.Text
          ref={this.textRef}
          style={[styles.text, this.textAnimation, this.props.style]}>
          {this.props.children}
        </Animated.Text>
      </View>
    );
  }

  private async startAnimation() {
    setTimeout(async () => {
      if (this.textRef.current) {
        const textLayout = await UserInterface.measureLayoutRelativeToWindow(this.textRef.current);
        const viewLayout = await UserInterface.measureLayoutRelativeToWindow(this);
        this.startAnimationImpl(textLayout.width - viewLayout.width, false);
      }
    }, 1000);
  }

  private startAnimationImpl(length: number, reverse: boolean) {
    if (length >= 0) {
      Animated.timing(this.initialLeft, {
        toValue: reverse ? 0.0 : -length,
        duration: length * 80,
        delay: 2000,
        easing: Animated.Easing.Linear(),
      }).start(() => {
        this.startAnimationImpl(length, !reverse);
      });
    }
  }
}

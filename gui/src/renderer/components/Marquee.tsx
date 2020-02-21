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

  private animationTimeout?: number;
  private animation?: Types.Animated.CompositeAnimation;

  public componentDidMount() {
    this.startAnimation();
  }

  public componentDidUpdate() {
    this.startAnimation();
  }

  public componentWillUnmount() {
    this.stopAnimation();
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

  private startAnimation() {
    this.stopAnimation();

    this.animationTimeout = setTimeout(async () => {
      if (this.textRef.current) {
        const textLayout = await UserInterface.measureLayoutRelativeToWindow(this.textRef.current);
        const viewLayout = await UserInterface.measureLayoutRelativeToWindow(this);
        this.startAnimationImpl(textLayout.width - viewLayout.width, false);
      }
    }, 1000);
  }

  private startAnimationImpl(length: number, reverse: boolean) {
    if (length >= 0) {
      this.animation = Animated.timing(this.initialLeft, {
        toValue: reverse ? 0.0 : -length,
        duration: length * 80,
        delay: 2000,
        easing: Animated.Easing.Linear(),
      });

      this.animation.start(({ finished }) => {
        if (finished) {
          this.startAnimationImpl(length, !reverse);
        }
      });
    }
  }

  private stopAnimation() {
    clearTimeout(this.animationTimeout);
    if (this.animation) {
      this.animation.stop();
    }
  }
}

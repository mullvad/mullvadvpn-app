// @flow
import * as React from 'react';

type Props = {
  children?: React.Node,
};

export default class PlatformWindow extends React.Component<Props> {
  render() {
    const chromeClass = ['window-chrome', 'window-chrome--' + process.platform];
    return <div className={chromeClass.join(' ')}>{this.props.children}</div>;
  }
}

// @flow
import * as React from 'react';
import { KeyboardAvoidingView } from 'react-native';

export default class PlatformWindow extends React.Component {
  props: {
    children: Array<React.Node> | React.Node
  };

  render() {
    return (
      <KeyboardAvoidingView behavior={'position'}>
        { this.props.children }
      </KeyboardAvoidingView>
    );
  }
}
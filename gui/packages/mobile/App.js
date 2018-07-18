// @flow

import * as React from 'react';
import { Component, Text, View, Platform, Styles } from 'reactxp';

const instructions = Platform.select({
  ios: 'Press Cmd+R to reload,\n' + 'Cmd+D or shake for dev menu',
  android: 'Double tap R on your keyboard to reload,\n' + 'Shake or press menu button for dev menu',
});

type Props = {};
export default class App extends Component<Props> {
  render() {
    return (
      <View style={styles.container}>
        <Text style={styles.welcome}>Welcome to React Native!</Text>
        <Text style={styles.instructions}>To get started, edit App.js</Text>
        <Text style={styles.instructions}>{instructions}</Text>
      </View>
    );
  }
}

const styles = {
  container: Styles.createViewStyle({
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#F5FCFF',
  }),
  welcome: Styles.createTextStyle({
    fontSize: 20,
    textAlign: 'center',
    margin: 10,
  }),
  instructions: Styles.createTextStyle({
    textAlign: 'center',
    color: '#333333',
    marginBottom: 5,
  }),
};

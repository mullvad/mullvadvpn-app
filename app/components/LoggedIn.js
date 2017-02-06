import React, { Component } from 'react';

export default class LoggedIn extends Component {
  render() {
    return (
      <div>
        <h2>Logged in as {this.props.user.username}</h2>
      </div>
    );
  }
}

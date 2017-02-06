/**
 * Declarative Tray implementation for React + Electron
 */

import React, { Component, PropTypes } from 'react';
import { remote } from 'electron';

const { Menu, MenuItem } = remote;

/**
 * Tray menu component
 * 
 * Example:
 * 
 * const tray = new remote.Tray('/path/to/icon');
 * 
 * return (
 *   <TrayMenu tray={tray}>
 *     <TrayItem label="Visit homepage" />
 *   </TrayMenu>
 * )
 */
export class TrayMenu extends Component {

  static childContextTypes = {
    menu: PropTypes.object.isRequired
  };

  static propTypes = {
    tray: PropTypes.object.isRequired,
    children: PropTypes.arrayOf(PropTypes.node).isRequired
  };

  _contextMenu = null;

  getChildContext() {
    return { menu: this._contextMenu };
  }

  componentDidMount() {
    this.props.tray.setContextMenu(this._contextMenu);
  }

  componentDidUpdate() {
    this.props.tray.setContextMenu(this._contextMenu);
  }

  render() {
    // create new menu during each rendering
    // see: https://github.com/electron/electron/issues/8598
    this._contextMenu = new Menu();

    return (
      <div>{this.props.children}</div>
    );
  }

}

/**
 * Submenu component
 * 
 * Example: 
 * 
 * <TrayMenu tray={this.props.handle}>
 *   <TraySubmenu label="Resources">
 *     <TrayItem label="Homepage" />
 *   </TraySubmenu>
 * </TrayMenu>
 * 
 */
export class TraySubmenu extends Component {

  static contextTypes = {
    menu: PropTypes.object.isRequired
  };

  static childContextTypes = {
    menu: PropTypes.object.isRequired
  };

  static propTypes = {
    children: PropTypes.arrayOf(PropTypes.node).isRequired
  };

  _contextMenu = null;

  getChildContext() {
    return { menu: this._contextMenu };
  }

  render() {
    // create new menu during each rendering
    // see: https://github.com/electron/electron/issues/8598
    this._contextMenu = new Menu();

    this.context.menu.append(new MenuItem({ ...this.props, submenu: this._contextMenu }));

    return (
      <div>{this.props.children}</div>
    );
  }

}

/**
 * Item component
 */
export class TrayItem extends Component {

  static contextTypes = {
    menu: PropTypes.object.isRequired
  };

  render() {
    this.context.menu.append(new MenuItem(this.props));
    return null;
  }

}

import { LocationScope } from './reducers';

export interface IUpdateLocaleAction {
  type: 'UPDATE_LOCALE';
  locale: string;
}

export interface IUpdateWindowArrowPositionAction {
  type: 'UPDATE_WINDOW_ARROW_POSITION';
  arrowPosition: number;
}

export interface IUpdateConnectionInfoOpenAction {
  type: 'TOGGLE_CONNECTION_PANEL';
}

export interface ISetLocationScopeAction {
  type: 'SET_LOCATION_SCOPE';
  scope: LocationScope;
}

export interface ISetWindowFocusedAction {
  type: 'SET_WINDOW_FOCUSED';
  focused: boolean;
}

export interface IAddScrollPosition {
  type: 'ADD_SCROLL_POSITION';
  path: string;
  scrollPosition: [number, number];
}

export interface IRemoveScrollPosition {
  type: 'REMOVE_SCROLL_POSITION';
  path: string;
}

export type UserInterfaceAction =
  | IUpdateLocaleAction
  | IUpdateWindowArrowPositionAction
  | IUpdateConnectionInfoOpenAction
  | ISetLocationScopeAction
  | ISetWindowFocusedAction
  | IAddScrollPosition
  | IRemoveScrollPosition;

function updateLocale(locale: string): IUpdateLocaleAction {
  return {
    type: 'UPDATE_LOCALE',
    locale,
  };
}

function updateWindowArrowPosition(arrowPosition: number): IUpdateWindowArrowPositionAction {
  return {
    type: 'UPDATE_WINDOW_ARROW_POSITION',
    arrowPosition,
  };
}

function toggleConnectionPanel(): IUpdateConnectionInfoOpenAction {
  return {
    type: 'TOGGLE_CONNECTION_PANEL',
  };
}

function setLocationScope(scope: LocationScope): ISetLocationScopeAction {
  return {
    type: 'SET_LOCATION_SCOPE',
    scope,
  };
}

function setWindowFocused(focused: boolean): ISetWindowFocusedAction {
  return {
    type: 'SET_WINDOW_FOCUSED',
    focused,
  };
}

function addScrollPosition(path: string, scrollPosition: [number, number]): IAddScrollPosition {
  return {
    type: 'ADD_SCROLL_POSITION',
    path,
    scrollPosition,
  };
}

function removeScrollPosition(path: string): IRemoveScrollPosition {
  return {
    type: 'REMOVE_SCROLL_POSITION',
    path,
  };
}

export default {
  updateLocale,
  updateWindowArrowPosition,
  toggleConnectionPanel,
  setLocationScope,
  setWindowFocused,
  addScrollPosition,
  removeScrollPosition,
};

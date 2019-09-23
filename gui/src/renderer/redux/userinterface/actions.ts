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

export type UserInterfaceAction =
  | IUpdateLocaleAction
  | IUpdateWindowArrowPositionAction
  | IUpdateConnectionInfoOpenAction
  | ISetLocationScopeAction;

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

export default { updateLocale, updateWindowArrowPosition, toggleConnectionPanel, setLocationScope };

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

export interface ISetShowWelcomeView {
  type: 'SET_SHOW_WELCOME_VIEW';
  showWelcomeView: boolean;
}

export type UserInterfaceAction =
  | IUpdateLocaleAction
  | IUpdateWindowArrowPositionAction
  | IUpdateConnectionInfoOpenAction
  | ISetLocationScopeAction
  | ISetShowWelcomeView;

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

function setShowWelcomeView(showWelcomeView: boolean): ISetShowWelcomeView {
  return {
    type: 'SET_SHOW_WELCOME_VIEW',
    showWelcomeView,
  };
}

export default {
  updateLocale,
  updateWindowArrowPosition,
  toggleConnectionPanel,
  setLocationScope,
  setShowWelcomeView,
};

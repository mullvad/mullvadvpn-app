import { LocationScope } from './reducers';

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
  | IUpdateWindowArrowPositionAction
  | IUpdateConnectionInfoOpenAction
  | ISetLocationScopeAction;

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

export default { updateWindowArrowPosition, toggleConnectionPanel, setLocationScope };

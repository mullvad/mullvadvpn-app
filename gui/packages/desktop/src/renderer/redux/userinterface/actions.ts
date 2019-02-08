export interface IUpdateWindowArrowPositionAction {
  type: 'UPDATE_WINDOW_ARROW_POSITION';
  arrowPosition: number;
}

export interface IUpdateConnectionInfoOpenAction {
  type: 'UPDATE_CONNECTION_INFO_OPEN';
  isOpen: boolean;
}

export type UserInterfaceAction =
  | IUpdateWindowArrowPositionAction
  | IUpdateConnectionInfoOpenAction;

function updateWindowArrowPosition(arrowPosition: number): IUpdateWindowArrowPositionAction {
  return {
    type: 'UPDATE_WINDOW_ARROW_POSITION',
    arrowPosition,
  };
}

function updateConnectionInfoOpen(isOpen: boolean): IUpdateConnectionInfoOpenAction {
  return {
    type: 'UPDATE_CONNECTION_INFO_OPEN',
    isOpen,
  };
}

export default { updateWindowArrowPosition, updateConnectionInfoOpen };

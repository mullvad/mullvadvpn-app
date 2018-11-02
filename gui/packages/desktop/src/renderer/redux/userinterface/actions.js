// @flow

export type UpdateWindowArrowPositionAction = {
  type: 'UPDATE_WINDOW_ARROW_POSITION',
  arrowPosition: number,
};

export type UpdateConnectionInfoOpenAction = {
  type: 'UPDATE_CONNECTION_INFO_OPEN',
  isOpen: boolean,
};

export type UserInterfaceAction = UpdateWindowArrowPositionAction | UpdateConnectionInfoOpenAction;

function updateWindowArrowPosition(arrowPosition: number): UpdateWindowArrowPositionAction {
  return {
    type: 'UPDATE_WINDOW_ARROW_POSITION',
    arrowPosition,
  };
}

function updateConnectionInfoOpen(isOpen: boolean): UpdateConnectionInfoOpenAction {
  return {
    type: 'UPDATE_CONNECTION_INFO_OPEN',
    isOpen,
  };
}

export default { updateWindowArrowPosition, updateConnectionInfoOpen };

// @flow

export type UpdateWindowArrowPositionAction = {
  type: 'UPDATE_WINDOW_ARROW_POSITION',
  arrowPosition: number,
};

export type WindowAction = UpdateWindowArrowPositionAction;

function updateWindowArrowPosition(arrowPosition: number): UpdateWindowArrowPositionAction {
  return {
    type: 'UPDATE_WINDOW_ARROW_POSITION',
    arrowPosition,
  };
}

export default { updateWindowArrowPosition };

import { handleActions } from 'redux-actions';

import actions from '../actions/user';

export default handleActions({
  [actions.loginChange]: (state, action) => {
    return { ...state, ...action.payload };
  }
}, {});

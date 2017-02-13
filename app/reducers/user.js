import { handleActions } from 'redux-actions';

import actions from '../actions/user';

const initialState = {
  account: ""
};

export default handleActions({
  [actions.loginChange]: (state, action) => {
    return { ...state, ...action.payload };
  }
}, initialState);

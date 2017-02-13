import { handleActions } from 'redux-actions';

import actions from '../actions/user';

import { LoginState } from '../constants';

const initialState = {
  account: '',
  status: LoginState.none
};

export default handleActions({
  [actions.loginChange]: (state, action) => {
    return { ...state, ...action.payload };
  }
}, initialState);

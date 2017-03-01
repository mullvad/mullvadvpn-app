import { handleActions } from 'redux-actions';
import actions from '../actions/user';
import { LoginState } from '../enums';

const initialState = {
  account: null,
  status: LoginState.none,
  error: null
};

export default handleActions({
  [actions.loginChange]: (state, action) => {
    return { ...state, ...action.payload };
  }
}, initialState);

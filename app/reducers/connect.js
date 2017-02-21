import { handleActions } from 'redux-actions';

import actions from '../actions/connect';

const initialState = {};

export default handleActions({ 
  [actions.connectionChange]: (state, action) => { 
    return { ...state, ...action.payload };
  } 
}, initialState);

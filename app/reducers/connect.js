import { handleActions } from 'redux-actions';

import actions from '../actions/connect';

const initialState = {};

export default handleActions({ test: (state) => { return state; } }, initialState);

import { useMemo } from 'react';
import { useDispatch } from 'react-redux';
import { ActionCreatorsMapObject, bindActionCreators } from 'redux';

export default function useActions<A, M extends ActionCreatorsMapObject<A>>(actionCreator: M) {
  const dispatch = useDispatch();
  const actions = useMemo(() => bindActionCreators(actionCreator, dispatch), [dispatch]);
  return actions;
}

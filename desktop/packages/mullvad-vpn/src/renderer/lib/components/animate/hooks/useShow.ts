import React from 'react';

import { useAnimateContext } from '../AnimateContext';

export const useShow = () => {
  const { present, show, setShow } = useAnimateContext();

  React.useEffect(() => {
    if (present && !show) {
      setShow(true);
    }
  }, [present, setShow, show]);
  return show;
};

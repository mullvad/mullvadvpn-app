import { useContext } from 'react';

import type { CustomContextReact, CustomContextValues } from './types';

const createUseCustomContext = <Values extends CustomContextValues>(
  Context: CustomContextReact<Values>,
) => {
  const useCustomContext = () => {
    const context = useContext(Context);

    if (typeof context === 'undefined') {
      throw new Error('Must be wrapped with <Provider /> before calling useContext!');
    }

    return context;
  };

  return useCustomContext;
};

export default createUseCustomContext;
